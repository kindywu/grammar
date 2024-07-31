use winnow::{
    ascii::{dec_int, float},
    combinator::alt,
    PResult, Parser,
};

fn main() {
    let inputs = vec![r#"19.9"#, r#"-199"#, r#"19.9"#, r#"-19.9"#];

    for input in inputs {
        let input = &mut (&*input);
        let ret = parse_number(input);
        println!("{:?}", ret);
    }
}

fn parse_number(input: &mut &str) -> PResult<Num> {
    alt((float.map(Num::Float), dec_int.map(Num::Int))).parse_next(input)
}

#[derive(Debug, Clone, PartialEq)]
enum Num {
    Float(f64),
    Int(i64),
}
