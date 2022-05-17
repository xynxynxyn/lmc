pub mod transform;

#[cfg(test)]
mod tests {
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
}
