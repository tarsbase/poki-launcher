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
use std::cmp::PartialEq;

use anyhow::{Error, Result};
use file_lock::FileLock;
use fuzzy_matcher::skim::fuzzy_match;
use rmp_serde as rmp;
// use serde_derive::{Deserialize, Serialize};
use serde::{de, Deserialize, Serialize};
use std::fmt::Debug;
use std::fs::File;
use std::io::Write as _;
use std::path::Path;
use std::process;
use std::slice::Iter;
use std::time::SystemTime;
use thiserror::Error;
use uuid::Uuid;

/// An apps database.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FrecencyDB<T: Debug + Clone + Serialize + DBItem + PartialEq> {
    /// The list of apps.
    items: Vec<Item<T>>,
    /// The reference time used in the ranking calculations.
    reference_time: f64,
    /// The half life of the app launches
    half_life: f32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Item<T: Debug + Clone + Serialize + DBItem + PartialEq> {
    pub item: T,
    score: f32,
    uuid: String,
}

struct Container<'a, T: Debug + Clone + Serialize + DBItem + PartialEq> {
    item: &'a Item<T>,
    sort_score: f32,
}

pub trait DBItem {
    fn get_sort_string(&self) -> &str;
}

#[allow(dead_code)]
impl<
        T: Debug + Clone + Serialize + de::DeserializeOwned + DBItem + PartialEq,
    > FrecencyDB<T>
{
    /// Create a new app.
    pub fn new(items: Vec<T>) -> Self {
        let items = items
            .into_iter()
            .map(|item| Item {
                item,
                score: 0.0,
                uuid: Uuid::new_v4().to_string(),
            })
            .collect();
        FrecencyDB {
            items,
            reference_time: current_time_secs(),
            // Half life of 3 days
            half_life: 60.0 * 60.0 * 24.0 * 3.0,
        }
    }

    /// Load database file.
    ///
    /// # Arguments
    ///
    /// * `path` - Location of the database file
    pub fn load(path: impl AsRef<Path>) -> Result<FrecencyDB<T>> {
        let path = path.as_ref().display().to_string();
        let lock = FileLock::lock(&path, true, false).map_err(|e| {
            Error::new(e).context(FrecencyDBError::FileOpen(path.to_owned()))
        })?;
        let apps: FrecencyDB<T> = rmp::from_read(&lock.file).map_err(|e| {
            Error::new(e).context(FrecencyDBError::ParseDB(path.to_owned()))
        })?;
        Ok(apps)
    }

    /// Save database file.
    ///
    /// # Arguments
    ///
    /// * `path` - Location of the database file
    pub fn save(&self, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref().display().to_string();
        let buf = rmp::to_vec(&self).expect("Failed to encode apps db");
        if Path::new(&path).exists() {
            let mut lock = FileLock::lock(&path, true, true).map_err(|e| {
                Error::new(e)
                    .context(FrecencyDBError::FileOpen(path.to_owned()))
            })?;
            lock.file.write_all(&buf).map_err(|e| {
                Error::new(e)
                    .context(FrecencyDBError::FileWrite(path.to_owned()))
            })?;
        } else {
            let mut file = File::create(&path).map_err(|e| {
                Error::new(e)
                    .context(FrecencyDBError::FileCreate(path.to_owned()))
            })?;
            file.write_all(&buf).map_err(|e| {
                Error::new(e)
                    .context(FrecencyDBError::FileWrite(path.to_owned()))
            })?;
        }
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
    ) -> Vec<&Item<T>> {
        let mut item_list = self
            .items
            .iter()
            .filter_map(|item| {
                match fuzzy_match(&item.item.get_sort_string(), &search) {
                    Some(score) if score > 0 => Some(Container {
                        item,
                        sort_score: item.score + score as f32,
                    }),
                    _ => None,
                }
            })
            .collect::<Vec<_>>();
        item_list.sort_by(|left, right| {
            right.sort_score.partial_cmp(&left.sort_score).unwrap()
        });
        if let Some(n) = num_items {
            item_list = item_list.into_iter().take(n).collect();
        }
        item_list.into_iter().map(|item| item.item).collect()
    }

    /// Increment to score for app `to_update` by 1 launch.
    pub fn update(&mut self, to_update: &Item<T>) {
        self.update_score(&to_update.uuid, 1.0);
    }

    /// Seconds elapsed since the reference time.
    fn secs_elapsed(&self) -> f32 {
        (current_time_secs() - self.reference_time) as f32
    }

    /// Update the score of an app.
    ///
    /// # Arguments
    ///
    /// * `uuid` - The uuid of the app to update.
    /// * `weight` - The amount to update to score by.
    pub fn update_score(&mut self, uuid: &str, weight: f32) {
        let elapsed = self.secs_elapsed();
        self.items
            .iter_mut()
            .find(|item| item.uuid == *uuid)
            .unwrap()
            .update_frecency(weight, elapsed, self.half_life);
    }

    /// Merge the apps from a re-scan into the database.
    ///
    /// * Apps in `self` that are not in `apps_to_merge` will be removed from `self`
    /// * Apps in `apps_to_merge` not in `self` will be added to `self`
    pub fn merge_new_entries(&mut self, mut items_to_merge: Vec<T>) {
        let items = std::mem::replace(&mut self.items, Vec::new());
        self.items = items
            .into_iter()
            .filter(|item| items_to_merge.contains(&item.item))
            .collect();
        items_to_merge = items_to_merge
            .into_iter()
            .filter(|item| {
                !self.items.iter().any(|l_item| l_item.item == *item)
            })
            .collect();
        for item in items_to_merge {
            self.items.push(Item::new(item));
        }
    }

    pub fn iter(&self) -> Iter<Item<T>> {
        self.items.iter()
    }
}

#[allow(dead_code)]
impl<
        T: Debug + Clone + Serialize + de::DeserializeOwned + DBItem + PartialEq,
    > Item<T>
{
    pub fn new(item: T) -> Self {
        Self {
            item,
            score: 0.0,
            uuid: Uuid::new_v4().to_string(),
        }
    }

    fn get_frecency(&self, elapsed: f32, half_life: f32) -> f32 {
        self.score / 2.0f32.powf(elapsed / half_life)
    }

    fn set_frecency(&mut self, new: f32, elapsed: f32, half_life: f32) {
        self.score = new * 2.0f32.powf(elapsed / half_life);
    }

    fn update_frecency(&mut self, weight: f32, elapsed: f32, half_life: f32) {
        self.set_frecency(
            self.get_frecency(elapsed, half_life) + weight,
            elapsed,
            half_life,
        );
    }
}

impl<
        T: Debug + Clone + Serialize + de::DeserializeOwned + DBItem + PartialEq,
    > PartialEq for Item<T>
{
    fn eq(&self, other: &Item<T>) -> bool {
        self.uuid == other.uuid
    }
}

/// Return the current time in seconds as a float
#[allow(dead_code)]
pub fn current_time_secs() -> f64 {
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
