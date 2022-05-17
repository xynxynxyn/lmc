use nom::{
    branch::alt,
    bytes::complete::tag,
    character::{complete::alphanumeric1, streaming::char},
    error::Error,
    sequence::{preceded, separated_pair},
    Finish, IResult, Parser,
};

#[derive(PartialEq, Clone, Debug, Hash)]
pub struct Formula {
    pub root_expr: Expr,
}

#[derive(PartialEq, Clone, Debug, Hash)]
pub enum Expr {
    True,
    False,
    Atomic(String),
    Or(Box<Expr>, Box<Expr>),
    And(Box<Expr>, Box<Expr>),
    Not(Box<Expr>),
    Next(Box<Expr>),
    Until(Box<Expr>, Box<Expr>),
    WeakUntil(Box<Expr>, Box<Expr>),
    Globally(Box<Expr>, Box<Expr>),
    Finally(Box<Expr>, Box<Expr>),
    Release(Box<Expr>, Box<Expr>),
    StrongRelease(Box<Expr>, Box<Expr>),
}

impl Formula {
    pub fn pnf(&self) -> Self {
        todo!()
    }

    pub fn parse(input: &str) -> Result<Self, Error<&str>> {
        Ok(Self {
            root_expr: Expr::parse(input).finish()?.1,
        })
    }
}

impl Expr {
    fn parse(input: &str) -> IResult<&str, Self> {
        alt((
            Expr::parse_false,
            Expr::parse_true,
            Expr::parse_not,
            Expr::parse_and,
            Expr::parse_or,
            Expr::parse_next,
            Expr::parse_finally,
            Expr::parse_globally,
            Expr::parse_until,
            Expr::parse_weak_until,
            Expr::parse_release,
            Expr::parse_strong_release,
            // parse identifier
            alphanumeric1.map(|s: &str| Expr::Atomic(s.to_string())),
        ))(input)
    }

    fn parse_false(input: &str) -> IResult<&str, Self> {
        tag("false").map(|_| Expr::False).parse(input)
    }

    fn parse_true(input: &str) -> IResult<&str, Self> {
        tag("true").map(|_| Expr::False).parse(input)
    }

    fn parse_not(input: &str) -> IResult<&str, Self> {
        preceded(tag("!"), Expr::parse.map(|e| Expr::Not(Box::new(e))))(input)
    }

    fn parse_next(input: &str) -> IResult<&str, Self> {
        preceded(tag("X "), Expr::parse.map(|e| Expr::Next(Box::new(e))))(input)
    }

    fn parse_and(input: &str) -> IResult<&str, Self> {
        preceded(
            tag("& "),
            separated_pair(Expr::parse, char(' '), Expr::parse)
                .map(|(e1, e2)| Expr::And(Box::new(e1), Box::new(e2))),
        )(input)
    }

    fn parse_or(input: &str) -> IResult<&str, Self> {
        preceded(
            tag("| "),
            separated_pair(Expr::parse, char(' '), Expr::parse)
                .map(|(e1, e2)| Expr::Or(Box::new(e1), Box::new(e2))),
        )(input)
    }

    fn parse_finally(input: &str) -> IResult<&str, Self> {
        preceded(
            tag("F "),
            separated_pair(Expr::parse, char(' '), Expr::parse)
                .map(|(e1, e2)| Expr::Finally(Box::new(e1), Box::new(e2))),
        )(input)
    }

    fn parse_globally(input: &str) -> IResult<&str, Self> {
        preceded(
            tag("G "),
            separated_pair(Expr::parse, char(' '), Expr::parse)
                .map(|(e1, e2)| Expr::Globally(Box::new(e1), Box::new(e2))),
        )(input)
    }

    fn parse_until(input: &str) -> IResult<&str, Self> {
        preceded(
            tag("U "),
            separated_pair(Expr::parse, char(' '), Expr::parse)
                .map(|(e1, e2)| Expr::Until(Box::new(e1), Box::new(e2))),
        )(input)
    }

    fn parse_weak_until(input: &str) -> IResult<&str, Self> {
        preceded(
            tag("W "),
            separated_pair(Expr::parse, char(' '), Expr::parse)
                .map(|(e1, e2)| Expr::WeakUntil(Box::new(e1), Box::new(e2))),
        )(input)
    }

    fn parse_release(input: &str) -> IResult<&str, Self> {
        preceded(
            tag("R "),
            separated_pair(Expr::parse, char(' '), Expr::parse)
                .map(|(e1, e2)| Expr::Release(Box::new(e1), Box::new(e2))),
        )(input)
    }

    fn parse_strong_release(input: &str) -> IResult<&str, Self> {
        preceded(
            tag("M "),
            separated_pair(Expr::parse, char(' '), Expr::parse)
                .map(|(e1, e2)| Expr::StrongRelease(Box::new(e1), Box::new(e2))),
        )(input)
    }
}
