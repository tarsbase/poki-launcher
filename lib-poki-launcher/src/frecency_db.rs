/***
 * This file is part of Poki Launcher.
 *
 * Poki Launcher is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * Poki Launcher is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with Poki Launcher.  If not, see <https://www.gnu.org/licenses/>.
 */

use log::*;

use anyhow::{Error, Result};
use fuzzy_matcher::skim::fuzzy_match;
use rmp_serde as rmp;
// use serde_derive::{Deserialize, Serialize};
use rusqlite::{params, Connection, OptionalExtension, NO_PARAMS};
use serde::{de, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::fmt::Debug;
use std::hash::Hash;
use std::hash::Hasher;
use std::marker::PhantomData;
use std::path::Path;
use std::process;
use std::time::SystemTime;
use thiserror::Error;

/// An apps database.
#[derive(Debug)]
pub struct FrecencyDB<T: DBItem> {
    /// The list of apps.
    conn: Connection,
    /// The reference time used in the ranking calculations.
    reference_time: f64,
    /// The half life of the app launches
    half_life: f64,
    _ph: PhantomData<T>,
}

pub struct Container<T: DBItem> {
    pub id: u64,
    pub item: T,
}

pub trait DBItem: Serialize + de::DeserializeOwned + Hash {
    fn get_sort_string(&self) -> &str;
}

fn update_frecency(
    score: f64,
    weight: f64,
    elapsed: f64,
    half_life: f64,
) -> f64 {
    let exp = 2.0f64.powf(elapsed / half_life);
    (score / exp + weight) * exp
}

macro_rules! table_def {
    ($input:expr, $tmp:expr) => {
        format!(
            "CREATE {} TABLE IF NOT EXISTS {} (
              id          INT PRIMARY KEY NOT NULL,
              score       REAL NOT NULL,
              sort_text   TEXT NOT NULL,
              data        BLOB NOT NULL
          );",
            if $tmp { "TEMPORARY" } else { "" },
            $input
        );
    };
}

#[allow(dead_code)]
impl<T: DBItem> FrecencyDB<T> {
    /// Create a new app.
    pub fn new(db_path: impl AsRef<Path>) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        conn.pragma_update(None, "temp_store", &"MEMORY")?;
        conn.execute(&table_def!("main", false), NO_PARAMS)?;
        conn.create_scalar_function("calc_score", 3, true, |ctx| {
            let f_score = ctx.get::<f64>(0)?;
            let text = ctx.get::<String>(1)?;
            let search = ctx.get::<String>(2)?;
            Ok(match fuzzy_match(&text, &search) {
                Some(score) if score > 0 => score as f64 + f_score,
                _ => 0.0,
            })
        })?;
        Ok(FrecencyDB {
            conn,
            reference_time: current_time_secs(),
            // Half life of 3 days
            half_life: 60.0 * 60.0 * 24.0 * 3.0,
            _ph: PhantomData::default(),
        })
    }

    /// Seconds elapsed since the reference time.
    fn secs_elapsed(&self) -> f64 {
        (current_time_secs() - self.reference_time)
    }

    /// Update the score of an app.
    ///
    /// # Arguments
    ///
    /// * `uuid` - The uuid of the app to update.
    /// * `weight` - The amount to update to score by.
    pub fn update_score(&mut self, id: u64) -> Result<()> {
        let score: f64 = self.conn.query_row(
            "SELECT * FROM main WHERE id = ?",
            &[id as i64],
            |row| row.get(1),
        )?;
        self.conn.execute(
             "UPDATE main SET score=? WHERE id=?;",
             params![
                 update_frecency(score, 1.0, self.secs_elapsed(), self.half_life,) as f64,
                 id as i64,
             ],
         )?;
        Ok(())
    }

    /// Merge the apps from a re-scan into the database.
    ///
    /// * Apps in `self` that are not in `apps_to_merge` will be removed from `self`
    /// * Apps in `apps_to_merge` not in `self` will be added to `self`
    pub fn merge_new_entries(
        &mut self,
        items_to_merge: &[impl DBItem],
    ) -> Result<()> {
        self.conn.execute_batch(&format!(
            "BEGIN;{}{}COMMIT;",
            table_def!("new", true),
            table_def!("tmp", false)
        ))?;
        let mut insert = self
             .conn
             .prepare("INSERT INTO new (id, score, sort_text, data) VALUES (?, 0.0, ?, ?);")?;
        for item in items_to_merge {
            let mut hasher = DefaultHasher::new();
            item.hash(&mut hasher);
            let id = hasher.finish() as i64;
            let sort_text = item.get_sort_string();
            let data = rmp::to_vec(&item)?;
            insert.execute(params![id, sort_text, data])?;
        }
        self.conn.execute_batch(
            "
             BEGIN;
             INSERT INTO tmp
             SELECT
                 new.id,
                 CASE WHEN main.score IS NOT NULL
                 THEN main.score
                 ELSE 0.0
                 END AS score,
                 CASE WHEN main.sort_text IS NOT NULL
                 THEN main.sort_text
                 ELSE new.sort_text
                 END AS sort_text,
                 CASE WHEN main.data IS NOT NULL
                 THEN main.data
                 ELSE new.data
                 END AS data
             FROM new LEFT OUTER JOIN main
             ON new.id = main.id;
             DROP TABLE main;
             DROP TABLE new;
             ALTER TABLE tmp RENAME TO main;
             COMMIT;",
        )?;
        Ok(())
    }

    /// Get the apps in rank order for a given search string.
    ///
    /// This ranks the apps both by frecency score and fuzzy search.
    // TODO Remove num_items
    pub fn get_ranked_list(
        &self,
        search: &str,
        num_items: Option<usize>,
    ) -> Result<Vec<Container<T>>> {
        let mut stmt = self.conn.prepare(
            "
         SELECT
         id, data, calc_score(score, sort_text, ?) as sort_score
         FROM main
         WHERE sort_score > 0
         ORDER BY sort_score DESC",
        )?;
        let item_iter = stmt.query_map(params![search], |row| {
            let id: i64 = row.get(0)?;
            let data: Vec<u8> = row.get(1)?;
            let obj: T = rmp::from_slice(&data).unwrap();
            Ok(Container {
                id: id as u64,
                item: obj,
            })
        })?;
        let res: Result<Vec<_>, _> = match num_items {
            Some(num) => item_iter.take(num).collect(),
            None => item_iter.collect(),
        };
        Ok(res?)
    }

    pub fn get_by_id(&self, id: u64) -> Result<Option<Container<T>>> {
        Ok(self
            .conn
            .query_row("SELECT * FROM main WHERE id = ?", &[id as i64], |row| {
                let id: i64 = row.get(0)?;
                let data: Vec<u8> = row.get(3)?;
                let obj: T = rmp::from_slice(&data).unwrap();
                Ok(Container {
                    id: id as u64,
                    item: obj,
                })
            })
            .optional()?)
    }
}

/// Return the current time in seconds as a float
fn current_time_secs() -> f64 {
    match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(n) => {
            (u128::from(n.as_secs()) * 1000 + u128::from(n.subsec_millis()))
                as f64
                / 1000.0
        }
        Err(e) => {
            // TODO handle this better
            error!("Invalid system time: {}", e);
            process::exit(1);
        }
    }
}

#[derive(Debug, Error)]
pub enum FrecencyDBError {
    #[error("Error opening apps database file {0}")]
    FileOpen(String),
    #[error("Error creating apps database file {0}")]
    FileCreate(String),
    #[error("Error writing to apps database file {0}")]
    FileWrite(String),
    #[error("Error parsing apps database file {0}")]
    ParseDB(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    impl DBItem for String {
        fn get_sort_string(&self) -> &str {
            self.as_str()
        }
    }

    // #[test]
    // fn big() {
    //     let mut db: FrecencyDB<String> = FrecencyDB::new("test.db").unwrap();
    //     let r1 = vec![
    //         "hello".to_owned(),
    //         "world".to_owned(),
    //         "hello world".to_owned(),
    //     ];
    //     db.merge_new_entries(&r1).unwrap();
    //     db.update_score(&r1[2]).unwrap();
    //     let out: Vec<String> = db.get_ranked_list("hello", None).unwrap();
    //     assert_eq!(out, vec!["test".to_owned()]);
    // }

    // #[test]
    // fn merge_new_entries_identical() {
    //     let items = vec!["hello".to_owned(), "world".to_owned()];
    //     let mut apps_db = FrecencyDB::new(items.clone());
    //     apps_db.merge_new_entries(items.clone());
    //     assert_eq!(items, apps_db.apps);
    // }

    // #[test]
    // fn merge_new_entries_remove() {
    //     let mut apps = vec![
    //         App::new(
    //             "Test1".to_owned(),
    //             "icon".to_owned(),
    //             "/bin/test".to_owned(),
    //             false,
    //         ),
    //         App::new(
    //             "Test2".to_owned(),
    //             "icon".to_owned(),
    //             "/bin/test".to_owned(),
    //             false,
    //         ),
    //     ];
    //     let mut apps_db = FrecencyDB::new(apps.clone());
    //     apps.remove(0);
    //     apps_db.merge_new_entries(apps.clone());
    //     assert_eq!(apps, apps_db.apps);
    // }

    // #[test]
    // fn merge_new_entries_add() {
    //     let mut apps = vec![App::new(
    //         "Test1".to_owned(),
    //         "icon".to_owned(),
    //         "/bin/test".to_owned(),
    //         false,
    //     )];
    //     let mut apps_db = FrecencyDB::new(apps.clone());
    //     apps.push(App::new(
    //         "Test2".to_owned(),
    //         "icon".to_owned(),
    //         "/bin/test".to_owned(),
    //         false,
    //     ));
    //     apps_db.merge_new_entries(apps.clone());
    //     assert_eq!(apps, apps_db.apps);
    // }
}
