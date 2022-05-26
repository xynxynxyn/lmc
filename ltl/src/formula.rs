use itertools::Itertools;
use std::{collections::BTreeSet, fmt::Display};

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::{complete::alphanumeric1, streaming::char},
    sequence::{preceded, separated_pair},
    IResult, Parser,
};

use crate::error::Error;

#[derive(Eq, PartialEq, Clone, Debug, Hash)]
pub struct Formula {
    pub root_expr: Expr,
}

#[derive(Eq, PartialEq, Clone, Debug, Hash, PartialOrd, Ord)]
pub enum Expr {
    True,
    False,
    Atomic(String),
    Not(Box<Expr>),
    Or(Box<Expr>, Box<Expr>),
    And(Box<Expr>, Box<Expr>),
    Next(Box<Expr>),
    Until(Box<Expr>, Box<Expr>),
    WeakUntil(Box<Expr>, Box<Expr>),
    Globally(Box<Expr>),
    Finally(Box<Expr>),
    Release(Box<Expr>, Box<Expr>),
    StrongRelease(Box<Expr>, Box<Expr>),
}

impl Formula {
    pub fn pnf(&self) -> Self {
        let mut root_expr = self.root_expr.simplify();
        loop {
            let new_root = root_expr.simplify();
            if new_root == root_expr {
                break;
            }

            root_expr = new_root;
        }

        Formula { root_expr }
    }

    pub fn parse(input: &str) -> Result<Self, crate::error::Error> {
        let root_expr = Expr::parse(input);
        let root_expr = root_expr.map_err(|e| {
            if e.is_incomplete() {
                Error::Incomplete(input.into())
            } else {
                Error::Parsing(e.to_string())
            }
        })?;
        if root_expr.0 != "" {
            return Err(Error::Leftover(input.into(), root_expr.0.into()));
        }

        Ok(Self {
            root_expr: root_expr.1,
        })
    }

    /// Compute the closure of the given formula (Every subformula and its negation)
    pub fn closure(&self) -> BTreeSet<Expr> {
        self.root_expr.closure()
    }

    pub fn elementary(&self) -> Vec<BTreeSet<Expr>> {
        // All non negated subformulae
        let closure = self.root_expr.basic_closure();
        let elementary = closure
            .clone()
            .into_iter()
            .powerset()
            .map(|s| {
                let mut s: BTreeSet<_> = s.into_iter().collect();
                for f in &closure {
                    if !s.contains(f) {
                        s.insert(Expr::Not(Box::new(f.clone())));
                    }
                }
                s
            })
            .filter(|s| {
                for e in &closure {
                    if !satisfies(s, e) {
                        return false;
                    }
                }

                true
            });
        elementary.collect()
    }

    pub fn alphabet(&self) -> BTreeSet<Expr> {
        let a = self.root_expr.alphabet();
        let mut b = a.clone();
        b.extend(a.into_iter().map(|expr| Expr::Not(Box::new(expr))));
        b
    }
}

fn satisfies(set: &BTreeSet<Expr>, expr: &Expr) -> bool {
    let exists = set.contains(expr) || set.contains(&Expr::Not(Box::new(expr.clone())));
    let satisfies = match expr {
        e @ Expr::False => !set.contains(e),
        e @ Expr::True => set.contains(e),
        e @ Expr::And(lhs, rhs) => {
            (set.contains(e) && set.contains(lhs) && set.contains(rhs))
                || (!set.contains(e) && !(set.contains(lhs) && set.contains(rhs)))
        }
        e @ Expr::Or(lhs, rhs) => {
            (set.contains(e) && (set.contains(lhs) || set.contains(rhs)))
                || (!set.contains(e) && !set.contains(lhs) && !set.contains(rhs))
        }
        e @ Expr::Until(lhs, rhs) => {
            (!set.contains(rhs) || set.contains(e))
                && (!(set.contains(e) && set.contains(&Expr::Not(Box::new(*rhs.clone()))))
                    || set.contains(lhs))
        }
        e @ Expr::Release(lhs, rhs) => {
            (!(set.contains(lhs) && set.contains(rhs)) || set.contains(e))
                && (!(set.contains(e) && set.contains(&Expr::Not(Box::new(*lhs.clone()))))
                    || set.contains(rhs))
        }
        _ => true,
    };
    exists && satisfies
}

impl Display for Formula {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.root_expr)
    }
}

impl Expr {
    pub fn alphabet(&self) -> BTreeSet<Expr> {
        match self {
            e @ Expr::True | e @ Expr::False | e @ Expr::Atomic(_) => BTreeSet::from([e.clone()]),
            Expr::Next(e) => e.alphabet(),
            Expr::Globally(e) => e.alphabet(),
            Expr::Finally(e) => e.alphabet(),
            Expr::Not(e) => e.alphabet(),
            Expr::And(lhs, rhs) => {
                let mut alphabet = BTreeSet::from(lhs.alphabet());
                alphabet.extend(rhs.alphabet());
                alphabet
            }
            Expr::Or(lhs, rhs) => {
                let mut alphabet = BTreeSet::from(lhs.alphabet());
                alphabet.extend(rhs.alphabet());
                alphabet
            }
            Expr::Until(lhs, rhs) => {
                let mut alphabet = BTreeSet::from(lhs.alphabet());
                alphabet.extend(rhs.alphabet());
                alphabet
            }
            Expr::WeakUntil(lhs, rhs) => {
                let mut alphabet = BTreeSet::from(lhs.alphabet());
                alphabet.extend(rhs.alphabet());
                alphabet
            }
            Expr::Release(lhs, rhs) => {
                let mut alphabet = BTreeSet::from(lhs.alphabet());
                alphabet.extend(rhs.alphabet());
                alphabet
            }
            Expr::StrongRelease(lhs, rhs) => {
                let mut alphabet = BTreeSet::from(lhs.alphabet());
                alphabet.extend(rhs.alphabet());
                alphabet
            }
        }
    }

    pub fn print_set(set: &BTreeSet<Self>) -> String {
        format!("{{{}}}", set.iter().sorted().join(", "))
    }

    fn basic_closure(&self) -> BTreeSet<Self> {
        match self {
            e @ Expr::True | e @ Expr::False => BTreeSet::from([e.clone()]),
            e @ Expr::Atomic(_) => BTreeSet::from([e.clone()]),
            Expr::Not(ex) => ex.basic_closure(),
            e @ Expr::Next(ex) => {
                let mut closure = BTreeSet::from([e.clone()]);
                closure.extend(ex.basic_closure());
                closure
            }
            e @ Expr::Globally(ex) => {
                let mut closure = BTreeSet::from([e.clone()]);
                closure.extend(ex.basic_closure());
                closure
            }
            e @ Expr::Finally(ex) => {
                let mut closure = BTreeSet::from([e.clone()]);
                closure.extend(ex.basic_closure());
                closure
            }
            e @ Expr::And(lhs, rhs) => {
                let mut closure = BTreeSet::from([e.clone()]);
                closure.extend(lhs.basic_closure());
                closure.extend(rhs.basic_closure());
                closure
            }
            e @ Expr::Or(lhs, rhs) => {
                let mut closure = BTreeSet::from([e.clone()]);
                closure.extend(lhs.basic_closure());
                closure.extend(rhs.basic_closure());
                closure
            }
            e @ Expr::Until(lhs, rhs) => {
                let mut closure = BTreeSet::from([e.clone()]);
                closure.extend(lhs.basic_closure());
                closure.extend(rhs.basic_closure());
                closure
            }
            e @ Expr::WeakUntil(lhs, rhs) => {
                let mut closure = BTreeSet::from([e.clone()]);
                closure.extend(lhs.basic_closure());
                closure.extend(rhs.basic_closure());
                closure
            }
            e @ Expr::Release(lhs, rhs) => {
                let mut closure = BTreeSet::from([e.clone()]);
                closure.extend(lhs.basic_closure());
                closure.extend(rhs.basic_closure());
                closure
            }
            e @ Expr::StrongRelease(lhs, rhs) => {
                let mut closure = BTreeSet::from([e.clone()]);
                closure.extend(lhs.basic_closure());
                closure.extend(rhs.basic_closure());
                closure
            }
        }
    }

    fn closure(&self) -> BTreeSet<Self> {
        let mut closure = self.basic_closure();
        let negated_closure = closure
            .clone()
            .into_iter()
            .map(|f| match f {
                e @ Expr::True | e @ Expr::False => e,
                _ => Expr::Not(Box::new(f)),
            })
            .collect::<BTreeSet<_>>();
        closure.extend(negated_closure);
        closure
    }

    fn simplify(&self) -> Self {
        match self {
            // Duality laws
            not_expr @ Expr::Not(ex) => match &**ex {
                Expr::True => Expr::False,
                Expr::False => Expr::True,
                Expr::Atomic(_) => not_expr.clone(),
                Expr::And(lhs, rhs) => Expr::Or(
                    Box::new(Expr::Not(Box::new(lhs.simplify()))),
                    Box::new(Expr::Not(Box::new(rhs.simplify()))),
                ),
                Expr::Or(lhs, rhs) => Expr::And(
                    Box::new(Expr::Not(Box::new(lhs.simplify()))),
                    Box::new(Expr::Not(Box::new(rhs.simplify()))),
                ),
                Expr::Next(ex) => Expr::Next(Box::new(Expr::Not(Box::new(ex.simplify())))),
                Expr::Finally(ex) => Expr::Globally(Box::new(Expr::Not(Box::new(ex.simplify())))),
                Expr::Globally(ex) => Expr::Finally(Box::new(Expr::Not(Box::new(ex.simplify())))),
                Expr::Until(lhs, rhs) => Expr::Release(
                    Box::new(Expr::Not(Box::new(lhs.simplify()))),
                    Box::new(Expr::Not(Box::new(rhs.simplify()))),
                ),
                Expr::Release(lhs, rhs) => Expr::Until(
                    Box::new(Expr::Not(Box::new(lhs.simplify()))),
                    Box::new(Expr::Not(Box::new(rhs.simplify()))),
                ),
                Expr::WeakUntil(lhs, rhs) => Expr::Until(
                    Box::new(Expr::Not(Box::new(rhs.simplify()))),
                    Box::new(Expr::And(
                        Box::new(Expr::Not(Box::new(lhs.simplify()))),
                        Box::new(Expr::Not(Box::new(rhs.simplify()))),
                    )),
                ),
                Expr::StrongRelease(lhs, rhs) => Expr::Release(
                    Box::new(Expr::Not(Box::new(rhs.simplify()))),
                    Box::new(Expr::Or(
                        Box::new(Expr::Not(Box::new(lhs.simplify()))),
                        Box::new(Expr::Not(Box::new(rhs.simplify()))),
                    )),
                ),
                Expr::Not(ex) => ex.simplify(),
            },
            e @ Expr::True | e @ Expr::False | e @ Expr::Atomic(_) => e.clone(),
            Expr::Next(e) => Expr::Next(Box::new(e.simplify())),
            Expr::And(lhs, rhs) => match (&**lhs, &**rhs) {
                (Expr::Next(le), Expr::Next(re)) => Expr::Next(Box::new(Expr::And(
                    Box::new(le.simplify()),
                    Box::new(re.simplify()),
                ))),
                (Expr::False, _) | (_, Expr::False) => Expr::False,
                (Expr::True, e @ _) | (e @ _, Expr::True) => e.simplify(),
                (lhs @ _, rhs @ Expr::Not(inner_r)) => {
                    if lhs == &**inner_r {
                        Expr::False
                    } else {
                        Expr::And(Box::new(lhs.simplify()), Box::new(rhs.simplify()))
                    }
                }
                (lhs @ Expr::Not(inner_l), rhs @ _) => {
                    if rhs == &**inner_l {
                        Expr::False
                    } else {
                        Expr::And(Box::new(lhs.simplify()), Box::new(rhs.simplify()))
                    }
                }
                (lhs @ _, rhs @ _) => Expr::And(Box::new(lhs.simplify()), Box::new(rhs.simplify())),
            },
            Expr::Or(lhs, rhs) => match (&**lhs, &**rhs) {
                (Expr::Next(le), Expr::Next(re)) => Expr::Next(Box::new(Expr::Or(
                    Box::new(le.simplify()),
                    Box::new(re.simplify()),
                ))),
                (Expr::True, _) | (_, Expr::True) => Expr::True,
                (Expr::False, e @ _) | (e @ _, Expr::False) => e.simplify(),
                (lhs @ _, rhs @ _) => Expr::Or(Box::new(lhs.simplify()), Box::new(rhs.simplify())),
            },
            Expr::Until(lhs, rhs) => {
                Expr::Until(Box::new(lhs.simplify()), Box::new(rhs.simplify()))
            }
            Expr::Release(lhs, rhs) => {
                Expr::Release(Box::new(lhs.simplify()), Box::new(rhs.simplify()))
            }
            // The ones below have to be changed to allowed symbols
            Expr::WeakUntil(lhs, rhs) => Expr::Release(
                Box::new(rhs.simplify()),
                Box::new(Expr::Or(Box::new(lhs.simplify()), Box::new(rhs.simplify()))),
            ),
            Expr::Globally(ex) => Expr::Release(Box::new(Expr::False), Box::new(ex.simplify())),
            Expr::Finally(ex) => Expr::Until(Box::new(Expr::True), Box::new(ex.simplify())),
            Expr::StrongRelease(lhs, rhs) => Expr::Until(
                Box::new(rhs.simplify()),
                Box::new(Expr::And(
                    Box::new(lhs.simplify()),
                    Box::new(rhs.simplify()),
                )),
            ),
        }
    }

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
        tag("true").map(|_| Expr::True).parse(input)
    }

    fn parse_not(input: &str) -> IResult<&str, Self> {
        preceded(tag("!"), Expr::parse.map(|e| Expr::Not(Box::new(e))))(input)
    }

    fn parse_next(input: &str) -> IResult<&str, Self> {
        preceded(tag("X "), Expr::parse.map(|e| Expr::Next(Box::new(e))))(input)
    }

    fn parse_globally(input: &str) -> IResult<&str, Self> {
        preceded(tag("G "), Expr::parse.map(|e| Expr::Globally(Box::new(e))))(input)
    }

    fn parse_finally(input: &str) -> IResult<&str, Self> {
        preceded(tag("F "), Expr::parse.map(|e| Expr::Finally(Box::new(e))))(input)
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

    fn fmt_braces(&self) -> String {
        match self {
            e @ Expr::Atomic(_)
            | e @ Expr::False
            | e @ Expr::True
            | e @ Expr::Not(_)
            | e @ Expr::Next(_) => e.to_string(),
            e @ _ => format!("({})", e),
        }
    }
}

impl Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let symbol = match self {
            Expr::Atomic(s) => s.clone(),
            Expr::True => "true".into(),
            Expr::False => "false".into(),
            Expr::Finally(ex) => format!("F {}", ex.fmt_braces()),
            Expr::Globally(ex) => format!("G {}", ex.fmt_braces()),
            Expr::Next(ex) => format!("X {}", ex.fmt_braces()),
            Expr::Not(ex) => format!("¬{}", ex.fmt_braces()),
            Expr::And(lhs, rhs) => format!("{} ∧ {}", lhs.fmt_braces(), rhs.fmt_braces()),
            Expr::Or(lhs, rhs) => format!("{} ∨ {}", lhs.fmt_braces(), rhs.fmt_braces()),
            Expr::Until(lhs, rhs) => format!("{} U {}", lhs.fmt_braces(), rhs.fmt_braces()),
            Expr::WeakUntil(lhs, rhs) => format!("{} W {}", lhs.fmt_braces(), rhs.fmt_braces()),
            Expr::Release(lhs, rhs) => format!("{} R {}", lhs.fmt_braces(), rhs.fmt_braces()),
            Expr::StrongRelease(lhs, rhs) => format!("{} M {}", lhs.fmt_braces(), rhs.fmt_braces()),
        };
        write!(f, "{}", symbol)
    }
}
