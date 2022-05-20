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
        while !root_expr.is_pnf() {
            let new_root = root_expr.simplify();
            if new_root == root_expr && !new_root.is_pnf() {
                panic!(
                    "Could not simplify {:?} any further but is not yet pnf",
                    new_root
                );
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
                for e in &self.closure() {
                    if !satisfies(s, e) {
                        return false;
                    }
                }

                true
            });
        elementary.collect()
    }

    pub fn alphabet(&self) -> BTreeSet<Expr> {
        self.root_expr.alphabet()
    }
}

fn satisfies(set: &BTreeSet<Expr>, expr: &Expr) -> bool {
    match expr {
        //not_e @ Expr::Not(e) => {
        //    let mut new_set = set.clone();
        //    new_set.insert(*e.clone());
        //    new_set.remove(not_e);
        //    !satisfies(&new_set, &*e)
        //}
        e @ Expr::And(lhs, rhs) => {
            !((set.contains(e) && (!set.contains(lhs) || !set.contains(rhs)))
                || (set.contains(lhs) && set.contains(rhs) && !set.contains(e)))
        }
        e @ Expr::Or(lhs, rhs) => {
            !((set.contains(e) && !set.contains(lhs) && !set.contains(rhs))
                || ((set.contains(lhs) || set.contains(rhs)) && !set.contains(e)))
        }
        e @ Expr::Until(lhs, rhs) => {
            !(set.contains(rhs) && !set.contains(e)
                || (set.contains(e) && !set.contains(rhs) && !set.contains(lhs)))
        }
        _ => true,
    }
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
            Expr::True | Expr::False => BTreeSet::from([Expr::True, Expr::False]),
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
                    Box::new(Expr::And(
                        Box::new(lhs.simplify()),
                        Box::new(Expr::Not(Box::new(rhs.simplify()))),
                    )),
                    Box::new(Expr::And(
                        Box::new(Expr::Not(Box::new(lhs.simplify()))),
                        Box::new(Expr::Not(Box::new(rhs.simplify()))),
                    )),
                ),
                Expr::StrongRelease(lhs, rhs) => Expr::WeakUntil(
                    Box::new(Expr::Not(Box::new(lhs.simplify()))),
                    Box::new(Expr::Not(Box::new(rhs.simplify()))),
                ),
                Expr::Not(ex) => ex.simplify(),
            },
            e @ Expr::True | e @ Expr::False | e @ Expr::Atomic(_) => e.clone(),
            Expr::Next(e) => Expr::Next(Box::new(e.simplify())),
            Expr::And(lhs, rhs) => Expr::And(Box::new(lhs.simplify()), Box::new(rhs.simplify())),
            Expr::Or(lhs, rhs) => Expr::Or(Box::new(lhs.simplify()), Box::new(rhs.simplify())),
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

    fn is_pnf(&self) -> bool {
        match self {
            Expr::Not(ex) => {
                if let Expr::Atomic(_) = **ex {
                    true
                } else {
                    false
                }
            }
            Expr::True | Expr::False | Expr::Atomic(_) => true,
            Expr::Next(e) => e.is_pnf(),
            Expr::And(lhs, rhs) => lhs.is_pnf() && rhs.is_pnf(),
            Expr::Or(lhs, rhs) => lhs.is_pnf() && rhs.is_pnf(),
            Expr::Until(lhs, rhs) => lhs.is_pnf() && rhs.is_pnf(),
            Expr::Release(lhs, rhs) => lhs.is_pnf() && rhs.is_pnf(),
            // Any other symbols are not allowed
            _ => false,
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
}

impl Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let symbol = match self {
            Expr::Atomic(s) => s.clone(),
            Expr::True => "true".into(),
            Expr::False => "false".into(),
            Expr::Finally(ex) => format!("F {}", ex),
            Expr::Globally(ex) => format!("G {}", ex),
            Expr::Next(ex) => format!("X {}", ex),
            Expr::Not(ex) => format!("!{}", ex),
            Expr::And(lhs, rhs) => format!("& {} {}", lhs, rhs),
            Expr::Or(lhs, rhs) => format!("| {} {}", lhs, rhs),
            Expr::Until(lhs, rhs) => format!("U {} {}", lhs, rhs),
            Expr::WeakUntil(lhs, rhs) => format!("W {} {}", lhs, rhs),
            Expr::Release(lhs, rhs) => format!("R {} {}", lhs, rhs),
            Expr::StrongRelease(lhs, rhs) => format!("M {} {}", lhs, rhs),
        };
        write!(f, "{}", symbol)
    }
}
