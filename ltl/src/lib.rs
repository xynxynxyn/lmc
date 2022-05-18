pub mod error;
pub mod transform;

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::transform::*;

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
}
