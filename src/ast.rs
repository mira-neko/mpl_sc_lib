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
    Func(String, Vec<Ast>),
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

impl Ast {
    fn program(input: &str) -> IResult<&str, Ast> {
        let (rest, value) = terminated(many0(Ast::instruction), eof)(input)?;
        Ok((rest, Ast::Root(value)))
    }

    fn instruction(input: &str) -> IResult<&str, Ast> {
        delimited(
            multispace0,
            alt((
                Ast::label,
                Ast::goto_if,
                Ast::goto_if_not,
                Ast::goto,
                Ast::_while,
                Ast::while_not,
                Ast::elihw,
                Ast::assign,
                Ast::func,
            )),
            newline,
        )(input)
    }

    fn exp(input: &str) -> IResult<&str, Ast> {
        alt((Ast::op, Ast::func, Ast::value, Ast::idnt))(input)
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
        let (rest, value) = pair(
            pair(
                alt((Ast::func, Ast::value, Ast::idnt)),
                alt((tag(" + "), tag(" - "), tag(" * "), tag(" / "), tag(" % "))),
            ),
            alt((Ast::func, Ast::value, Ast::idnt)),
        )(input)?;
        Ok((
            rest,
            Ast::Func(
                match value.0 .1 {
                    " + " => "add",
                    " - " => "sub",
                    " * " => "mul",
                    " / " => "div",
                    " % " => "mod",
                    _ => unreachable!(),
                }
                .to_string(),
                vec![value.0 .0, value.1],
            ),
        ))
    }

    fn func(input: &str) -> IResult<&str, Ast> {
        let (rest, value) = pair(
            alphanumeric1,
            delimited(
                tag("("),
                separated_list0(terminated(tag(","), multispace0), Ast::exp),
                tag(")"),
            ),
        )(input)?;
        Ok((rest, Ast::Func(value.0.to_string(), value.1)))
    }
}
