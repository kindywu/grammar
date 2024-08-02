use std::collections::HashMap;

use anyhow::{anyhow, Result};
use winnow::{
    ascii::{dec_int, float, multispace0},
    combinator::{alt, delimited, opt, separated, separated_pair},
    token::take_until,
    PResult, Parser,
};

use winnow::combinator::seq;

fn main() -> Result<()> {
    let s = r#"{
        "name": "John Doe",
        "age": 30,
        "is_student": false,
        "marks": [90.0, -80.0, 85, 11e-1],
        "address": {
            "city": "New York",
            "zip": 10001
        }
    }"#;

    let json_value = parse_json(s).map_err(|e| anyhow!("{}", e))?;

    println!("{json_value:#?}");

    Ok(())
}

fn parse_json(mut s: &str) -> PResult<JsonValue> {
    let input = &mut s;
    parse_value(input)
}

fn parse_null(input: &mut &str) -> PResult<()> {
    "null".value(()).parse_next(input)
}

fn parse_bool(input: &mut &str) -> PResult<bool> {
    alt(("true", "false")).parse_to().parse_next(input)
}

fn parse_num(input: &mut &str) -> PResult<Num> {
    let (remain, _): (&str, i64) = dec_int.parse_peek(*input)?;
    if !remain.starts_with('.') {
        let num: i64 = dec_int(input)?;
        Ok(Num::Int(num))
    } else {
        let num: f64 = float(input)?;
        Ok(Num::Float(num))
    }
}

fn parse_string(input: &mut &str) -> PResult<String> {
    let str = delimited('"', take_until(0.., '"'), '"').parse_next(input)?;
    Ok(str.to_owned())
}

fn parse_value(input: &mut &str) -> PResult<JsonValue> {
    alt((
        parse_null.value(JsonValue::Null),
        parse_bool.map(JsonValue::Bool),
        parse_string.map(JsonValue::String),
        parse_num.map(JsonValue::Number),
        parse_array.map(JsonValue::Array),
        parse_object.map(JsonValue::Object),
    ))
    .parse_next(input)
}

#[macro_export]
macro_rules! sep_with_space {
    ($delimiter:expr) => {
        seq!(multispace0, $delimiter, multispace0)
    };
    ($delimiter1:expr, $($delimiter:expr),+) => {
        seq!(
            multispace0,
            opt($delimiter1),
            multispace0,
            sep_with_space!(@inner $($delimiter),+)
        )
    };
    (@inner $delimiter:expr) => {
        seq!($delimiter, multispace0)
    };
    (@inner $delimiter1:expr, $($delimiter:expr),+) => {
        seq!(
            opt($delimiter1),
            multispace0,
            sep_with_space!(@inner $($delimiter),+)
        )
    };
}

fn parse_array(input: &mut &str) -> PResult<Vec<JsonValue>> {
    let sep1 = sep_with_space!("[");
    let sep2 = sep_with_space!(",", "]");
    let sep_comma = sep_with_space!(",");
    let parse_value = separated(0.., parse_value, sep_comma);
    delimited(sep1, parse_value, sep2).parse_next(input)
}

fn parse_object(input: &mut &str) -> PResult<HashMap<String, JsonValue>> {
    let sep1 = sep_with_space!("{");
    let sep2 = sep_with_space!(",", "}");
    let sep_comma = sep_with_space!(",");
    let sep_colon = sep_with_space!(":");

    let parse_kv_pair = separated_pair(parse_string, sep_colon, parse_value);
    let parse_kv = separated(1.., parse_kv_pair, sep_comma);
    delimited(sep1, parse_kv, sep2).parse_next(input)
}

#[derive(Debug, Clone, PartialEq)]
enum Num {
    Int(i64),
    Float(f64),
}

#[derive(Debug, Clone, PartialEq)]
enum JsonValue {
    Null,
    Bool(bool),
    Number(Num),
    String(String),
    Array(Vec<JsonValue>),
    Object(HashMap<String, JsonValue>),
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_parse_null() -> PResult<()> {
        let input = "null";
        parse_null(&mut (&*input))?;

        Ok(())
    }

    #[test]
    fn test_parse_bool() -> PResult<()> {
        let input = "true";
        let result = parse_bool(&mut (&*input))?;
        assert!(result);

        let input = "false";
        let result = parse_bool(&mut (&*input))?;
        assert!(!result);

        Ok(())
    }

    #[test]
    fn test_parse_num() -> PResult<()> {
        let input = "99 ";
        assert_eq!(parse_num(&mut (&*input)), Ok(Num::Int(99)));

        let input = "-199";
        assert_eq!(parse_num(&mut (&*input)), Ok(Num::Int(-199)));

        let input = "199.8";
        assert_eq!(parse_num(&mut (&*input)), Ok(Num::Float(199.8)));

        let input = "-199.8";
        assert_eq!(parse_num(&mut (&*input)), Ok(Num::Float(-199.8)));

        Ok(())
    }

    #[test]
    fn test_parse_string() -> PResult<()> {
        let input = r#""hello world""#;
        let result = parse_string(&mut (&*input))?;
        assert_eq!(result, "hello world");

        let input = r#""""#;
        let result = parse_string(&mut (&*input))?;
        assert_eq!(result, "");

        Ok(())
    }

    #[test]
    fn test_parse_value() -> PResult<()> {
        let input = r#"null"#;
        let result = parse_value(&mut (&*input))?;
        assert_eq!(result, JsonValue::Null);

        let input = r#"true"#;
        let result = parse_value(&mut (&*input))?;
        assert_eq!(result, JsonValue::Bool(true));

        let input = r#""hello world""#;
        let result = parse_value(&mut (&*input))?;
        assert_eq!(result, JsonValue::String("hello world".to_string()));

        let input = r#"199"#;
        let result = parse_value(&mut (&*input))?;
        assert_eq!(result, JsonValue::Number(Num::Int(199)));

        let input = r#"199.99"#;
        let result = parse_value(&mut (&*input))?;
        assert_eq!(result, JsonValue::Number(Num::Float(199.99)));
        Ok(())
    }

    #[test]
    fn test_parse_object() -> PResult<()> {
        let inputs = vec![
            r#"{
        "name":"kindywu",
        "age":30,
        "score":30.4
        }"#,
            r#"{
        "name":"kindywu",
        "age":30,
        "score":30.4,
        }"#,
        ];

        let mut map = HashMap::new();
        map.insert("name".to_string(), JsonValue::String("kindywu".to_owned()));
        map.insert("age".to_string(), JsonValue::Number(Num::Int(30)));
        map.insert("score".to_string(), JsonValue::Number(Num::Float(30.4)));

        for input in inputs {
            let result = parse_object(&mut (&*input))?;
            assert_eq!(result, map);
        }

        Ok(())
    }

    #[test]
    fn test_parse_array() -> PResult<()> {
        let input = r#"[]"#;
        let result = parse_array(&mut (&*input))?;
        assert_eq!(Vec::<JsonValue>::new(), result);

        let inputs = vec![
            r#" ["kindy", 44, 33.33, true, null] "#,
            r#" ["kindy", 44, 33.33, true, null, ] "#,
        ];

        let mut arr = Vec::new();
        arr.push(JsonValue::String("kindy".to_string()));
        arr.push(JsonValue::Number(Num::Int(44)));
        arr.push(JsonValue::Number(Num::Float(33.33)));
        arr.push(JsonValue::Bool(true));
        arr.push(JsonValue::Null);

        for input in inputs {
            let result = parse_array(&mut (&*input))?;
            assert_eq!(result, arr);
        }

        Ok(())
    }
}
