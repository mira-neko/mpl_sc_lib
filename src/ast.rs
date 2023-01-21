use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{alphanumeric1, multispace0, newline},
    combinator::eof,
    multi::many0,
    multi::separated_list0,
    number::complete::double,
    sequence::{delimited, pair, preceded, terminated},
    IResult,
};

pub(super) enum Ast {
    Root(Vec<Ast>),
    Value(f64),
    Idnt(String),
    Assign(String, Box<Ast>),
    Input,
    Print(Vec<Ast>),
    Add(Box<Ast>, Box<Ast>),
    Sub(Box<Ast>, Box<Ast>),
    Mul(Box<Ast>, Box<Ast>),
    Div(Box<Ast>, Box<Ast>),
    Mod(Box<Ast>, Box<Ast>),
    Max(Box<Ast>, Box<Ast>),
    Min(Box<Ast>, Box<Ast>),
    Swap(String, String),
    Label(String),
    Goto(String),
    GotoIf(String, Box<Ast>),
    GotoIfNot(String, Box<Ast>),
    While(Box<Ast>),
    WhileNot(Box<Ast>),
    Elihw,
}

impl From<&str> for Ast {
    fn from(s: &str) -> Ast {
        Ast::program(s).unwrap().1
    }
}

macro_rules! op {
    ($name:ident, $op:literal, $op_name:ident) => {
        fn $name(input: &str) -> IResult<&str, Ast> {
            let (rest, value) = pair(
                terminated(alt((delimited(tag("("), Ast::op, tag(")")), Ast::func, Ast::value, Ast::idnt)), tag($op)),
                alt((delimited(tag("("), Ast::op, tag(")")), Ast::func, Ast::value, Ast::idnt)),
            )(input)?;
            Ok((rest, Ast::$op_name(Box::new(value.0), Box::new(value.1))))
        }
    };
}

macro_rules! assign_op {
    ($name:ident, $op:literal, $op_name:ident) => {
        fn $name(input: &str) -> IResult<&str, Ast> {
            let (rest, value) = pair(terminated(alphanumeric1, tag($op)), Ast::exp)(input)?;
            Ok((
                rest,
                Ast::Assign(
                    value.0.to_string(),
                    Box::new(Ast::$op_name(
                        Box::new(Ast::Idnt(value.0.to_string())),
                        Box::new(value.1)
                    )),
                ),
            ))
        }
    };
}

impl Ast {
    fn program(input: &str) -> IResult<&str, Ast> {
        let (rest, value) = terminated(many0(Ast::instruction), eof)(input)?;
        Ok((rest, Ast::Root(value)))
    }

    fn instruction(input: &str) -> IResult<&str, Ast> {
        delimited(
            multispace0,
            alt((
                Ast::swap,
                Ast::label,
                Ast::goto_if,
                Ast::goto_if_not,
                Ast::goto,
                Ast::_while,
                Ast::while_not,
                Ast::elihw,
                Ast::assign,
                Ast::assign_op,
                Ast::func,
            )),
            newline,
        )(input)
    }

    fn exp(input: &str) -> IResult<&str, Ast> {
        alt((Ast::op, Ast::func, Ast::value, Ast::idnt))(input)
    }

    fn swap(input: &str) -> IResult<&str, Ast> {
        let (rest, value) = pair(
            preceded(tag("swap "), alphanumeric1),
            preceded(tag(" and "), alphanumeric1),
        )(input)?;
        Ok((rest, Ast::Swap(value.0.to_string(), value.1.to_string())))
    }

    fn label(input: &str) -> IResult<&str, Ast> {
        let (rest, value) = terminated(alphanumeric1, tag(":"))(input)?;
        Ok((rest, Ast::Label(value.to_string())))
    }

    fn goto(input: &str) -> IResult<&str, Ast> {
        let (rest, value) = preceded(tag("goto "), alphanumeric1)(input)?;
        Ok((rest, Ast::Goto(value.to_string())))
    }

    fn goto_if(input: &str) -> IResult<&str, Ast> {
        let (rest, value) = delimited(
            tag("goto "),
            pair(alphanumeric1, preceded(tag(" if "), Ast::exp)),
            tag(" = 0"),
        )(input)?;
        Ok((rest, Ast::GotoIf(value.0.to_string(), Box::new(value.1))))
    }

    fn goto_if_not(input: &str) -> IResult<&str, Ast> {
        let (rest, value) = delimited(
            tag("goto "),
            pair(alphanumeric1, preceded(tag(" if "), Ast::exp)),
            tag(" != 0"),
        )(input)?;
        Ok((rest, Ast::GotoIfNot(value.0.to_string(), Box::new(value.1))))
    }

    fn _while(input: &str) -> IResult<&str, Ast> {
        let (rest, value) = delimited(tag("while "), Ast::exp, tag(" = 0"))(input)?;
        Ok((rest, Ast::While(Box::new(value))))
    }

    fn while_not(input: &str) -> IResult<&str, Ast> {
        let (rest, value) = delimited(tag("while "), Ast::exp, tag(" != 0"))(input)?;
        Ok((rest, Ast::WhileNot(Box::new(value))))
    }

    fn elihw(input: &str) -> IResult<&str, Ast> {
        let (rest, _) = tag("elihw")(input)?;
        Ok((rest, Ast::Elihw))
    }

    fn assign(input: &str) -> IResult<&str, Ast> {
        let (rest, value) = pair(terminated(alphanumeric1, tag(" = ")), Ast::exp)(input)?;
        Ok((rest, Ast::Assign(value.0.to_string(), Box::new(value.1))))
    }

    fn value(input: &str) -> IResult<&str, Ast> {
        let (rest, value) = double(input)?;
        Ok((rest, Ast::Value(value)))
    }

    fn idnt(input: &str) -> IResult<&str, Ast> {
        let (rest, value) = alphanumeric1(input)?;
        Ok((rest, Ast::Idnt(value.to_string())))
    }

    fn op(input: &str) -> IResult<&str, Ast> {
        alt((
            Ast::op_add,
            Ast::op_sub,
            Ast::op_mul,
            Ast::op_div,
            Ast::op_mod,
        ))(input)
    }

    op!(op_add, " + ", Add);
    op!(op_sub, " - ", Sub);
    op!(op_mul, " * ", Mul);
    op!(op_div, " / ", Div);
    op!(op_mod, " % ", Mod);

    fn assign_op(input: &str) -> IResult<&str, Ast> {
        alt((
            Ast::assign_op_add,
            Ast::assign_op_sub,
            Ast::assign_op_mul,
            Ast::assign_op_div,
            Ast::assign_op_mod,
        ))(input)
    }

    assign_op!(assign_op_add, " += ", Add);
    assign_op!(assign_op_sub, " -= ", Sub);
    assign_op!(assign_op_mul, " *= ", Mul);
    assign_op!(assign_op_div, " /= ", Div);
    assign_op!(assign_op_mod, " %= ", Mod);

    fn func(input: &str) -> IResult<&str, Ast> {
        alt((Ast::inp, Ast::print, Ast::max, Ast::min))(input)
    }

    fn inp(input: &str) -> IResult<&str, Ast> {
        let (rest, _) = tag("input()")(input)?;
        Ok((rest, Ast::Input))
    }

    fn print(input: &str) -> IResult<&str, Ast> {
        let (rest, value) = delimited(
            tag("print("),
            separated_list0(tag(", "), Ast::exp),
            tag(")"),
        )(input)?;
        Ok((rest, Ast::Print(value)))
    }

    fn max(input: &str) -> IResult<&str, Ast> {
        let (rest, (inner1, inner2)) = delimited(
            tag("max("),
            pair(terminated(Ast::exp, tag(", ")), Ast::exp),
            tag(")"),
        )(input)?;
        Ok((rest, Ast::Max(Box::new(inner1), Box::new(inner2))))
    }

    fn min(input: &str) -> IResult<&str, Ast> {
        let (rest, (inner1, inner2)) = delimited(
            tag("min("),
            pair(terminated(Ast::exp, tag(", ")), Ast::exp),
            tag(")"),
        )(input)?;
        Ok((rest, Ast::Min(Box::new(inner1), Box::new(inner2))))
    }
}
