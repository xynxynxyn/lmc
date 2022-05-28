use itertools::Itertools;
use std::{cmp::Ordering, collections::BTreeSet, fmt::Display};

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

#[derive(Eq, PartialEq, Clone, Debug, Hash, Ord, PartialOrd)]
pub enum Expr {
    True,
    False,
    Not(Box<Expr>),
    Atomic(String),
    Next(Box<Expr>),
    Globally(Box<Expr>),
    Finally(Box<Expr>),
    Or(Box<Expr>, Box<Expr>),
    And(Box<Expr>, Box<Expr>),
    Until(Box<Expr>, Box<Expr>),
    WeakUntil(Box<Expr>, Box<Expr>),
    Release(Box<Expr>, Box<Expr>),
    StrongRelease(Box<Expr>, Box<Expr>),
}

impl Formula {
    pub fn pnf(&self) -> Self {
        Formula {
            root_expr: self.root_expr.pnf(),
        }
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
        let closure = self.root_expr.subformula();
        let elementary = closure
            .clone()
            .into_iter()
            .powerset()
            .map(|s| {
                let mut s: BTreeSet<_> = s.into_iter().collect();
                for f in &closure {
                    if let Expr::False | Expr::True = f {
                        continue;
                    }
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
        b.extend(
            a.into_iter()
                .filter(|expr| match expr {
                    Expr::True | Expr::False => false,
                    _ => true,
                })
                .map(|expr| Expr::Not(Box::new(expr))),
        );
        b
    }
}

fn satisfies(set: &BTreeSet<Expr>, expr: &Expr) -> bool {
    let exists = set.contains(expr) || set.contains(&expr.negated());
    let satisfies = match expr {
        e @ Expr::False => return !set.contains(e),
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
                && (!(set.contains(e) && set.contains(&rhs.negated())) || set.contains(lhs))
        }
        e @ Expr::Release(lhs, rhs) => {
            (!(set.contains(lhs) && set.contains(rhs)) || set.contains(e))
                && (!(set.contains(e) && set.contains(&lhs.negated())) || set.contains(rhs))
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
    fn negated(&self) -> Self {
        match self {
            Expr::True => Expr::False,
            Expr::False => Expr::True,
            Expr::Not(e) => *e.clone(),
            _ => Expr::Not(Box::new(self.clone())),
        }
    }

    pub fn alphabet(&self) -> BTreeSet<Expr> {
        match self {
            Expr::True | Expr::False => BTreeSet::new(),
            e @ Expr::Atomic(_) => BTreeSet::from([e.clone()]),
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

    fn pnf(&self) -> Self {
        let mut root_expr = self.simplify();
        loop {
            let new_root = root_expr.simplify();
            if new_root == root_expr {
                break;
            }
            root_expr = new_root;
        }
        root_expr
    }

    pub fn print_set(set: &BTreeSet<Self>) -> String {
        format!(
            "{{{}}}",
            set.iter().sorted_by(|r, s| Expr::cmp(r, s)).join(", ")
        )
    }

    fn subformula(&self) -> BTreeSet<Self> {
        match self {
            e @ Expr::False | e @ Expr::True | e @ Expr::Atomic(_) => BTreeSet::from([e.clone()]),
            Expr::Not(ex) => ex.subformula(),
            e @ Expr::Next(ex) => {
                let mut closure = BTreeSet::from([e.clone()]);
                closure.extend(ex.subformula());
                closure
            }
            e @ Expr::Globally(ex) => {
                let mut closure = BTreeSet::from([e.clone()]);
                closure.extend(ex.subformula());
                closure
            }
            e @ Expr::Finally(ex) => {
                let mut closure = BTreeSet::from([e.clone()]);
                closure.extend(ex.subformula());
                closure
            }
            e @ Expr::And(lhs, rhs) => {
                let mut closure = BTreeSet::from([e.clone()]);
                closure.extend(lhs.subformula());
                closure.extend(rhs.subformula());
                closure
            }
            e @ Expr::Or(lhs, rhs) => {
                let mut closure = BTreeSet::from([e.clone()]);
                closure.extend(lhs.subformula());
                closure.extend(rhs.subformula());
                closure
            }
            e @ Expr::Until(lhs, rhs) => {
                let mut closure = BTreeSet::from([e.clone()]);
                closure.extend(lhs.subformula());
                closure.extend(rhs.subformula());
                closure
            }
            e @ Expr::WeakUntil(lhs, rhs) => {
                let mut closure = BTreeSet::from([e.clone()]);
                closure.extend(lhs.subformula());
                closure.extend(rhs.subformula());
                closure
            }
            e @ Expr::Release(lhs, rhs) => {
                let mut closure = BTreeSet::from([e.clone()]);
                closure.extend(lhs.subformula());
                closure.extend(rhs.subformula());
                closure
            }
            e @ Expr::StrongRelease(lhs, rhs) => {
                let mut closure = BTreeSet::from([e.clone()]);
                closure.extend(lhs.subformula());
                closure.extend(rhs.subformula());
                closure
            }
        }
    }

    fn closure(&self) -> BTreeSet<Self> {
        let mut closure = self.subformula();
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
}

// Formatting
impl Expr {
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

    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Expr::True | Expr::False, Expr::True | Expr::False) => Ordering::Equal,
            (Expr::True | Expr::False, _) => Ordering::Less,

            (Expr::Not(a), Expr::Not(b)) => a.cmp(b),
            (Expr::Not(a), b @ _) => {
                if let Ordering::Equal = a.as_ref().cmp(b) {
                    Ordering::Greater
                } else {
                    a.as_ref().cmp(b)
                }
            }
            (a @ _, Expr::Not(b)) => {
                if let Ordering::Equal = a.cmp(b) {
                    Ordering::Less
                } else {
                    a.cmp(b)
                }
            }

            (Expr::Atomic(a), Expr::Atomic(b)) => a.cmp(b),
            (Expr::Atomic(_), Expr::True | Expr::False) => Ordering::Greater,
            (Expr::Atomic(_), _) => Ordering::Less,

            (Expr::Next(a), Expr::Next(b)) => a.cmp(b),
            (Expr::Next(_), Expr::True | Expr::False | Expr::Atomic(_)) => Ordering::Greater,
            (Expr::Next(_), _) => Ordering::Less,

            (Expr::Globally(a), Expr::Globally(b)) => a.cmp(b),
            (Expr::Globally(_), Expr::True | Expr::False | Expr::Atomic(_) | Expr::Next(_)) => {
                Ordering::Greater
            }
            (Expr::Globally(_), _) => Ordering::Less,

            (Expr::Finally(a), Expr::Finally(b)) => a.cmp(b),
            (
                Expr::Finally(_),
                Expr::True | Expr::False | Expr::Atomic(_) | Expr::Next(_) | Expr::Globally(_),
            ) => Ordering::Greater,
            (Expr::Finally(_), _) => Ordering::Less,

            (Expr::Or(a1, a2), Expr::Or(b1, b2)) => {
                if let Ordering::Equal = a1.cmp(b1) {
                    a2.cmp(b2)
                } else {
                    a1.cmp(b2)
                }
            }
            (
                Expr::Or(_, _),
                Expr::True
                | Expr::False
                | Expr::Atomic(_)
                | Expr::Next(_)
                | Expr::Globally(_)
                | Expr::Finally(_),
            ) => Ordering::Greater,
            (Expr::Or(_, _), _) => Ordering::Less,

            (Expr::And(a1, a2), Expr::And(b1, b2)) => {
                if let Ordering::Equal = a1.cmp(b1) {
                    a2.cmp(b2)
                } else {
                    a1.cmp(b2)
                }
            }
            (
                Expr::And(_, _),
                Expr::True
                | Expr::False
                | Expr::Atomic(_)
                | Expr::Next(_)
                | Expr::Globally(_)
                | Expr::Finally(_)
                | Expr::Or(_, _),
            ) => Ordering::Greater,
            (Expr::And(_, _), _) => Ordering::Less,

            (Expr::Until(a1, a2), Expr::Until(b1, b2)) => {
                if let Ordering::Equal = a1.cmp(b1) {
                    a2.cmp(b2)
                } else {
                    a1.cmp(b2)
                }
            }
            (
                Expr::Until(_, _),
                Expr::WeakUntil(_, _) | Expr::Release(_, _) | Expr::StrongRelease(_, _),
            ) => Ordering::Less,
            (Expr::Until(_, _), _) => Ordering::Greater,

            (Expr::WeakUntil(a1, a2), Expr::WeakUntil(b1, b2)) => {
                if let Ordering::Equal = a1.cmp(b1) {
                    a2.cmp(b2)
                } else {
                    a1.cmp(b2)
                }
            }
            (Expr::WeakUntil(_, _), Expr::Release(_, _) | Expr::StrongRelease(_, _)) => {
                Ordering::Less
            }
            (Expr::WeakUntil(_, _), _) => Ordering::Greater,

            (Expr::Release(a1, a2), Expr::Release(b1, b2)) => {
                if let Ordering::Equal = a1.cmp(b1) {
                    a2.cmp(b2)
                } else {
                    a1.cmp(b2)
                }
            }
            (Expr::Release(_, _), Expr::StrongRelease(_, _)) => Ordering::Less,
            (Expr::Release(_, _), _) => Ordering::Greater,

            (Expr::StrongRelease(a1, a2), Expr::StrongRelease(b1, b2)) => {
                if let Ordering::Equal = a1.cmp(b1) {
                    a2.cmp(b2)
                } else {
                    a1.cmp(b2)
                }
            }
            (Expr::StrongRelease(_, _), _) => Ordering::Greater,
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

// Parsing
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

#[cfg(test)]
mod test {
    use super::*;
    // Expression tests
    #[test]
    pub fn simple_pnf() {
        let cases = vec![
            ("!& a b", "| !a !b"),
            ("& true a", "a"),
            ("& false a", "false"),
            ("!| a b", "& !a !b"),
            ("F a", "U true a"),
            ("G a", "R false a"),
            ("W a b", "R b | a b"),
            ("M a b", "U b & a b"),
            ("!!a", "a"),
            ("!!!a", "!a"),
            ("!X a", "X !a"),
            ("!F a", "R false !a"),
            ("!G a", "U true !a"),
            ("!U a b", "R !a !b"),
            ("!R a b", "U !a !b"),
        ];

        for (input, expected) in cases {
            assert_eq!(
                Formula::parse(input).unwrap().pnf(),
                Formula::parse(expected).unwrap()
            );
        }
    }
}
