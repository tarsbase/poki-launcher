mod parser {
    use pest_derive::Parser;

    #[derive(Parser, Debug)]
    #[grammar = "desktop_file.pest"]
    pub(super) struct DesktopFileParser;
}

#[derive(Debug, Default)]
struct Entry {
    name: String,
    icon: String,
    exec: String
}

#[derive(Debug)]
enum EntryData {
    Name(String),
    Icon(String),
    Exec(String),
}

#[derive(Debug, Eq, PartialEq)]
enum DesktopValue<'a> {
    Header(&'a str),
    Param((&'a str, &'a str)),
    EOI,
}

pub(super) fn parse_desktop_file(file: &str) {
    use self::parser::{DesktopFileParser, Rule};
    use pest::Parser;
    use pest::iterators::Pair;
    use from_pest::FromPest;
    use std::mem;

    let parse_tree = DesktopFileParser::parse(Rule::desktop_file, file).unwrap().next().unwrap();
//    dbg!(parse_tree);

    fn parse_value(pair: Pair<Rule>) -> DesktopValue {
        match pair.as_rule() {
            Rule::header => {
                DesktopValue::Header(pair.into_inner().next().unwrap().as_str())
            }
            Rule::param => {
                let mut inner = pair.into_inner();
                let key = inner.next().unwrap().as_str();
                let value = inner.next().unwrap().as_str();
                DesktopValue::Param((key, value))
            }
            Rule::EOI => DesktopValue::EOI,
            _ => unreachable!(),
        }
    }

    let values: Vec<DesktopValue> = parse_tree.into_inner().map(parse_value).collect();
//    dbg!(values);

    let mut entries = Vec::new();
    let mut start_new = true;
    let mut current: Entry = Default::default();
    for val in values {
        match val {
            DesktopValue::Header(_) => {
                if !start_new {
                    entries.push(mem::replace(&mut current, Default::default()));
                    start_new = false;
                }
                start_new = true;
            }
            DesktopValue::Param((key, value)) => {
                start_new = false;
                match key {
                    "Name" => current.name = String::from(value),
                    "Icon" => current.icon = String::from(value),
                    "Exec" => current.exec = String::from(value),
                    _ => {}
                }
            }
            _ => {}
        }
    }
    entries.push(current);
    dbg!(entries);
}


#[cfg(test)]
mod test {
    use super::parse_desktop_file;

    #[test]
    fn parser() {
        let input = r#"
[Desktop Entry]
Name=CrossCode
Icon=steam_icon_368340
Exec=steam steam://rungameid/368340
"#;
        parse_desktop_file(input);
    }
}

//Name=CrossCode
//Comment=Play this game on Steam
//Exec=steam steam://rungameid/368340
//Icon=steam_icon_368340
//Terminal=false
//Type=Application
//Categories=Game;
