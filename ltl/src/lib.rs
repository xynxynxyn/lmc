pub mod error;
pub mod formula;

#[cfg(test)]
mod tests {
    use std::collections::{BTreeSet, HashMap};

    use itertools::Itertools;

    use crate::formula::*;

    #[test]
    fn parse1() {
        let input = "false";
        assert_eq!(Formula::parse(&input).unwrap().root_expr, Expr::False);
    }

    #[test]
    fn parse2() {
        let input = "& a b";
        assert_eq!(
            Formula::parse(&input).unwrap().root_expr,
            Expr::And(
                Box::new(Expr::Atomic("a".into())),
                Box::new(Expr::Atomic("b".into()))
            )
        )
    }

    #[test]
    fn parse3() {
        let input = "U & a b !c";
        assert_eq!(
            Formula::parse(&input).unwrap().root_expr,
            Expr::Until(
                Box::new(Expr::And(
                    Box::new(Expr::Atomic("a".into())),
                    Box::new(Expr::Atomic("b".into()))
                )),
                Box::new(Expr::Not(Box::new(Expr::Atomic("c".into()))))
            )
        )
    }

    #[test]
    fn invalid_parse() {
        assert!(Formula::parse("U & a b c d").is_err())
    }

    #[test]
    fn pnf() {
        let values = HashMap::from([
            ("!& a b", "| !a !b"),
            ("!& | a c b", "| & !a !c !b"),
            ("!X a", "X !a"),
            ("W a b", "R b | a b"),
            ("G a", "R false a"),
            ("F a", "U true a"),
        ]);
        for (l, r) in values {
            let lhs = Formula::parse(l).unwrap().pnf();
            let rhs = Formula::parse(r).unwrap();
            assert_eq!(
                lhs, rhs,
                "'{}' in PNF should be '{}' but is '{}'",
                l, rhs, lhs
            );
        }
    }

    #[test]
    fn closure() {
        let values = HashMap::from([
            (
                Formula::parse("& a b").unwrap(),
                BTreeSet::from([
                    Expr::And(
                        Box::new(Expr::Atomic("a".into())),
                        Box::new(Expr::Atomic("b".into())),
                    ),
                    Expr::Not(Box::new(Expr::And(
                        Box::new(Expr::Atomic("a".into())),
                        Box::new(Expr::Atomic("b".into())),
                    ))),
                    Expr::Atomic("a".into()),
                    Expr::Not(Box::new(Expr::Atomic("a".into()))),
                    Expr::Atomic("b".into()),
                    Expr::Not(Box::new(Expr::Atomic("b".into()))),
                ]),
            ),
            (
                Formula::parse("!a").unwrap(),
                BTreeSet::from([
                    Expr::Atomic("a".into()),
                    Expr::Not(Box::new(Expr::Atomic("a".into()))),
                ]),
            ),
        ]);

        for (l, r) in values {
            assert_eq!(l.closure(), r, "input: {}", l);
        }
    }

    #[test]
    fn elementary1() {
        let formula = Formula::parse("& a b").unwrap();
        let elementary_sets = Formula::parse("& a b").unwrap().elementary();
        let should_contain = vec![
            BTreeSet::from([
                Expr::Atomic("a".into()),
                Expr::Atomic("b".into()),
                Expr::And(
                    Box::new(Expr::Atomic("a".into())),
                    Box::new(Expr::Atomic("b".into())),
                ),
            ]),
            BTreeSet::from([
                Expr::Not(Box::new(Expr::Atomic("a".into()))),
                Expr::Atomic("b".into()),
                Expr::Not(Box::new(Expr::And(
                    Box::new(Expr::Atomic("a".into())),
                    Box::new(Expr::Atomic("b".into())),
                ))),
            ]),
            BTreeSet::from([
                Expr::Not(Box::new(Expr::Atomic("b".into()))),
                Expr::Atomic("a".into()),
                Expr::Not(Box::new(Expr::And(
                    Box::new(Expr::Atomic("a".into())),
                    Box::new(Expr::Atomic("b".into())),
                ))),
            ]),
            BTreeSet::from([
                Expr::Not(Box::new(Expr::Atomic("a".into()))),
                Expr::Not(Box::new(Expr::Atomic("b".into()))),
                Expr::Not(Box::new(Expr::And(
                    Box::new(Expr::Atomic("a".into())),
                    Box::new(Expr::Atomic("b".into())),
                ))),
            ]),
        ];

        assert!(
            elementary_sets.len() == should_contain.len(),
            "{}",
            elementary_sets
                .into_iter()
                .map(|s| Expr::print_set(&s))
                .join(", ")
        );
        for e in &elementary_sets {
            assert!(
                should_contain.contains(e),
                "{:?} != {:?}",
                elementary_sets,
                should_contain
            );
        }
    }
}
