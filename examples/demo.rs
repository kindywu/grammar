use winnow::ascii::multispace0;

use winnow::combinator::opt;
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

fn main() {
    let input = r#" ["kindy", "abc", "" ,  ] "#;
    let input = &mut (&*input);

    let sep1 = sep_with_space!("[");
    // let sep2 = seq!(multispace0, opt(","), multispace0, "]", multispace0);
    let sep2 = sep_with_space!(",", "]");
    let sep_comma = sep_with_space!(",");

    let parse_value = separated(0.., parse_string, sep_comma);

    // let arr: Vec<String> = parse_value.parse_next(input).unwrap();

    let arr: Vec<String> = delimited(sep1, parse_value, sep2)
        .parse_next(input)
        .unwrap();

    println!("{:?}", arr);
}

fn parse_string(input: &mut &str) -> PResult<String> {
    let str = delimited('"', take_until(0.., '"'), '"').parse_next(input)?;
    Ok(str.to_owned())
}
