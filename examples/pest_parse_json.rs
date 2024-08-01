use std::collections::HashMap;

use anyhow::{anyhow, Result};
use pest::{iterators::Pair, Parser};
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "examples/json.pest"]
struct JsonParser;

#[allow(unused)]
#[derive(Debug, PartialEq)]
enum JsonValue {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<JsonValue>),
    Object(HashMap<String, JsonValue>),
}

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

    if let Some(pair) = JsonParser::parse(Rule::object, s)?.next() {
        println!("{:#?}", pair);
        let value = parse_value(pair);
        println!("{:#?}", value);
    } else {
        println!("json has no value")
    };

    Ok(())
}

fn parse_value(pair: Pair<Rule>) -> Result<JsonValue> {
    let value = match pair.as_rule() {
        Rule::null => JsonValue::Null,
        Rule::bool => JsonValue::Bool(pair.as_str().parse()?),
        Rule::number => JsonValue::Number(pair.as_str().trim().parse()?),
        Rule::chars => JsonValue::String(pair.as_str().to_string()),
        Rule::value => parse_inner(pair)?,
        Rule::array => JsonValue::Array(parse_array(pair)?),
        Rule::object => JsonValue::Object(parse_object(pair)?),
        r => {
            println!("rule: {:#?}", r);
            unreachable!()
        }
    };
    Ok(value)
}

fn parse_object(pair: Pair<Rule>) -> Result<HashMap<String, JsonValue>> {
    let map = pair
        .into_inner()
        .map(|pair| {
            let mut inner = pair.into_inner(); //pair

            let key = inner
                .next()
                .map(|p| p.as_str().to_string())
                .ok_or_else(|| anyhow!("key is null"))?;

            let value = inner
                .next()
                .map(|p| parse_value(p))
                .ok_or_else(|| anyhow!("value is null"))??;

            Ok((key, value))
        })
        .collect();
    map
}

fn parse_array(pair: Pair<Rule>) -> Result<Vec<JsonValue>> {
    pair.into_inner().map(parse_value).collect()
}

fn parse_inner(pair: Pair<Rule>) -> Result<JsonValue> {
    let inner = pair
        .into_inner()
        .next()
        .ok_or_else(|| anyhow!("inner is null"))?;
    parse_value(inner)
}

// cargo nextest run --example pest
#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    #[test]
    fn pest_parse_null_should_work() -> Result<()> {
        let input = "null";
        let parsed = JsonParser::parse(Rule::null, input)?.next().unwrap();
        let result = parse_value(parsed)?;
        assert_eq!(JsonValue::Null, result);

        Ok(())
    }

    #[test]
    fn pest_parse_bool_should_work() -> Result<()> {
        let input = "true";
        let parsed = JsonParser::parse(Rule::bool, input)?.next().unwrap();
        let result = parse_value(parsed)?;
        assert_eq!(JsonValue::Bool(true), result);

        Ok(())
    }

    #[test]
    fn pest_parse_number_should_work() -> Result<()> {
        let input = "-199";
        let parsed = JsonParser::parse(Rule::number, input)?.next().unwrap();
        let result = parse_value(parsed)?;
        assert_eq!(JsonValue::Number(-199.0), result);

        let input = "11e-1";
        let parsed = JsonParser::parse(Rule::number, input)?.next().unwrap();
        let result = parse_value(parsed)?;
        assert_eq!(JsonValue::Number(1.1), result);
        Ok(())
    }

    #[test]
    fn pest_parse_string_should_work() -> Result<()> {
        let input = r#""name""#;
        let parsed = JsonParser::parse(Rule::string, input)?.next().unwrap();
        let result = parse_value(parsed)?;
        assert_eq!(JsonValue::String("name".to_string()), result);

        Ok(())
    }

    #[test]
    #[should_panic]
    fn pest_parse_string_should_not_work() -> () {
        let input = r#""1name""#;
        let parsed = JsonParser::parse(Rule::string, input)
            .unwrap()
            .next()
            .unwrap();
        let result = parse_value(parsed).unwrap();
        assert_eq!(JsonValue::String(input.to_string()), result);
    }

    #[test]
    fn pest_parse_array_should_work() -> Result<()> {
        let input = r#"["name",1,true,null]"#;
        let parsed = JsonParser::parse(Rule::array, input)?.next().unwrap();
        let result = parse_value(parsed)?;
        assert_eq!(
            JsonValue::Array(vec![
                JsonValue::String("name".to_string()),
                JsonValue::Number(1.0),
                JsonValue::Bool(true),
                JsonValue::Null,
            ]),
            result
        );

        Ok(())
    }

    #[test]
    fn pest_parse_object_should_work() -> Result<()> {
        let input = r#"{ "name" : "kindy" , "age" : 18 }"#;
        let parsed = JsonParser::parse(Rule::object, input)?.next().unwrap();
        let result = parse_value(parsed)?;
        let mut map = HashMap::new();
        map.insert("name".to_string(), JsonValue::String("kindy".to_string()));
        map.insert("age".to_string(), JsonValue::Number(18.0));
        assert_eq!(JsonValue::Object(map), result);

        Ok(())
    }
}
