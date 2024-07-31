use winnow::ascii::multispace0;

use winnow::{
    combinator::{delimited, separated, seq},
    token::take_until,
    PResult, Parser,
};

#[macro_export]
macro_rules! sep_with_space {
    ($delimiter:expr) => {
        seq!(multispace0, $delimiter, multispace0)
    };
}

fn main() {
    let input = r#"["kindy", "abc"]"#;
    let input = &mut (&*input);

    let sep1 = sep_with_space!("[");
    let sep2 = sep_with_space!("]");
    let sep_comma = sep_with_space!(",");
    let parse_value = separated(0.., parse_string, sep_comma);
    let arr: Vec<String> = delimited(sep1, parse_value, sep2)
        .parse_next(input)
        .unwrap();

    println!("{arr:?}")
}

fn parse_string(input: &mut &str) -> PResult<String> {
    let str = delimited('"', take_until(0.., '"'), '"').parse_next(input)?;
    Ok(str.to_owned())
}
