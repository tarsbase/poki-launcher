mod parser {
    use pest_derive::Parser;

    #[derive(Parser, Debug)]
    #[grammar = "desktop_file.pest"]
    pub(super) struct DesktopFileParser;
}

mod omnom {
    use nom::{
        named, map, do_parse, opt, many0, terminated, separated_pair, map_res, take_while, char, call,
        IResult,
        bytes::complete::take_while,
        sequence::delimited,
        combinator::map_res,
        character::complete::{char, alphanumeric1 as alphanumeric, multispace0 as multispace, space0 as space}
    };

    use std::str;
    use std::collections::HashMap;

    fn header(i: &[u8]) -> IResult<&[u8], &str> {
        map_res(delimited(char('['), take_while(|c| c != b']'),
                          char(']')), str::from_utf8)(i)
    }

    fn complete_byte_slice_to_str(s: &[u8]) -> Result<&str, str::Utf8Error> {
        str::from_utf8(s)
    }

    named!(pub key_value<&[u8], (&str, &str)>,
        do_parse!(
            key: map_res!(alphanumeric, complete_byte_slice_to_str)
            >>  opt!(space)
            >>  char!('=')
            >>  opt!(space)
            >>
            value: map_res!(
               take_while!(call!(|c| c != b'\n')),
               complete_byte_slice_to_str
            )
            >>
            (key, value)
        )
    );


    named!(pub keys_and_values<&[u8], HashMap<&str, &str> >,
        map!(
            many0!(terminated!(key_value, opt!(multispace))),
            |vec: Vec<_>| vec.into_iter().collect()
        )
    );

    named!(header_and_keys<&[u8],(&str,HashMap<&str,&str>)>,
        do_parse!(
        header: header         >>
                  opt!(multispace) >>
        keys: keys_and_values      >>
        (header, keys)
        )
    );

    named!(pub headers<&[u8], HashMap<&str, HashMap<&str,&str> > >,
        map!(
            many0!(
            separated_pair!(
                header,
                    opt!(multispace),
                    map!(
                        many0!(terminated!(key_value, opt!(multispace))),
                        |vec: Vec<_>| vec.into_iter().collect()
                    )
                )
            ),
            |vec: Vec<_>| vec.into_iter().collect()
        )
    );

}

mod ast {
    use pest_ast::*;
    use pest::Span;
    use super::parser::Rule;

    fn span_into_str(span: Span) -> &str {
        span.as_str()
    }

//    #[derive(Debug, FromPest)]
//    #[pest_ast(rule(Rule::entry))]
//    pub struct Entry<'a> {
//        #[pest_ast(outer(with(span_into_str)))]
//        key: &'a str,
//        #[pest_ast(outer(with(span_into_str)))]
//        value: &'a str,
//    }


}

#[derive(Debug)]
struct Entry {
    name: String,
    data: Vec<EntryData>,
}

#[derive(Debug)]
enum EntryData {
    Name(String),
    Icon(String),
    Exec(String),
}

#[derive(Debug)]
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

    let mut parse_tree = DesktopFileParser::parse(Rule::desktop_file, file).unwrap().next().unwrap();
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
//                DesktopValue::Param(
//                    pair.into_inner()
//                        .map(|pair| {
//                            let mut inner_rules = pair.into_inner();
//                            let key = inner_rules
//                                .next()
//                                .unwrap()
//                                .into_inner()
//                                .next()
//                                .unwrap()
//                                .as_str();
//                            let value = inner_rules
//                                .next()
//                                .unwrap()
//                                .into_inner()
//                                .next()
//                                .unwrap()
//                                .as_str();
//                            (key, value)
//                        })
//                        .collect()
//                )
            }
            Rule::EOI => DesktopValue::EOI,
            Rule::header_val => unreachable!(),
            Rule::key => unreachable!(),
            Rule::value => unreachable!(),
            Rule::desktop_file => unreachable!(),
            _ => unreachable!(),
        }
    }
    let values: Vec<DesktopValue> = parse_tree.into_inner().map(parse_value).collect();
    dbg!(values);

//    let ast = Vec::new();
//    for item in parse_tree.into_inner() {
//        dbg!(item);
//        match item.as_rule() {
//            Rule::header => {
//
//            }
//            Rule::param => {
//
//            }
//            _ => {}
//        }
//    }

//    let mut parse_tree = DesktopFileParser::parse(Rule::entry, file).unwrap().next().unwrap();
//    dbg!(parse_tree);
//    let ast: Entry = Entry::from_pest(&mut parse_tree).unwrap();
//    dbg!(ast);

}


#[cfg(test)]
mod test {
    use super::parse_desktop_file;
    use super::omnom::{headers, key_value, keys_and_values};

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
