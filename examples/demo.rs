use winnow::{
    ascii::{dec_int, float},
    PResult, Parser,
};

fn main() {
    let inputs = vec![r#"199 "#, r#"-199"#, r#"19.9"#, r#"-19.9"#, "11e-2"];

    for input in inputs {
        let input = &mut (&*input);
        let ret = parse_num(input);
        println!("{:?}", ret);
    }
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

#[derive(Debug, Clone, PartialEq)]
enum Num {
    Float(f64),
    Int(i64),
}
