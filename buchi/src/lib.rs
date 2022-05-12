pub mod nba;

#[cfg(test)]
mod test {
    use super::nba::*;
    #[test]
    pub fn two_state_nba() {
        let mut nba = Buchi::new();
        let s1 = State::new("s1".into());
        let s2 = State::new("s2".into());
        let w = Word::new("a".into());

        nba.add_transition(&s1, &s2, &w);
        nba.add_transition(&s2, &s1, &w);

        assert!(nba.transitions(&s1).unwrap().get(&w).unwrap().contains(&s2));
        assert!(nba.transitions(&s2).unwrap().get(&w).unwrap().contains(&s1));
    }

    #[test]
    pub fn three_state_nba() {
        let mut nba = Buchi::new();
        let s1 = State::new("s1".into());
        let s2 = State::new("s2".into());
        let s3 = State::new("s3".into());
        let a = Word::new("a".into());
        let b = Word::new("a".into());

        nba.add_transition(&s1, &s2, &a);
        nba.add_transition(&s1, &s3, &b);
        nba.add_transition(&s3, &s2, &b);

        let s1_trans = nba.transitions(&s1).unwrap();
        let s2_trans = nba.transitions(&s2).unwrap();
        let s3_trans = nba.transitions(&s3).unwrap();

        assert!(s1_trans.get(&a).unwrap().contains(&s2));
        assert!(s1_trans.get(&b).unwrap().contains(&s3));
        assert!(s2_trans.is_empty());
        assert!(s3_trans.get(&b).unwrap().contains(&s2));
    }

    #[test]
    pub fn tarjan() {
        let mut nba = Buchi::new();
        let a = State::new("a".into());
        let b = State::new("b".into());
        let e = State::new("e".into());
        let c = State::new("c".into());
        let d = State::new("d".into());
        let f = State::new("f".into());
        let g = State::new("g".into());
        let h = State::new("h".into());
        let x = Word::new("x".into());
        let y = Word::new("y".into());
        let z = Word::new("z".into());

        nba.add_transition(&a, &b, &x);
        nba.add_transition(&b, &e, &x);
        nba.add_transition(&e, &a, &x);
        nba.add_transition(&b, &f, &x);
        nba.add_transition(&b, &c, &x);
        nba.add_transition(&e, &f, &x);
        nba.add_transition(&c, &d, &x);
        nba.add_transition(&d, &c, &x);
        nba.add_transition(&d, &h, &x);
        nba.add_transition(&h, &d, &x);
        nba.add_transition(&c, &g, &y);
        nba.add_transition(&h, &g, &z);
        nba.add_transition(&g, &f, &x);
        nba.add_transition(&f, &g, &x);

        let components = nba.tarjans();
        assert!(components.len() == 3, "{:?}", components);
    }

    #[test]
    pub fn verify_complex() {
        let mut nba = Buchi::new();
        let a = State::new("a".into());
        let b = State::new("b".into());
        let e = State::new("e".into());
        let c = State::new("c".into());
        let d = State::new("d".into());
        let f = State::new("f".into());
        let g = State::new("g".into());
        let h = State::new("h".into());
        let x = Word::new("x".into());
        let y = Word::new("y".into());
        let z = Word::new("z".into());

        nba.add_transition(&a, &b, &x);
        nba.add_transition(&b, &e, &x);
        nba.add_transition(&e, &a, &x);
        nba.add_transition(&b, &f, &x);
        nba.add_transition(&b, &c, &x);
        nba.add_transition(&e, &f, &x);
        nba.add_transition(&c, &d, &x);
        nba.add_transition(&d, &c, &x);
        nba.add_transition(&d, &h, &x);
        nba.add_transition(&h, &d, &x);
        nba.add_transition(&c, &g, &x);
        nba.add_transition(&h, &g, &x);
        nba.add_transition(&g, &f, &y);
        nba.add_transition(&f, &g, &z);

        nba.initial_state(&a);
        nba.accepting_state(&f);

        let trace = nba.verify();
        assert!(trace.is_err(), "{:?}", trace);
        let trace = trace.unwrap_err();
        assert!(trace.omega_words.contains(&y), "{}", trace);
        assert!(trace.omega_words.contains(&z), "{}", trace)
    }

    #[test]
    pub fn verify_simple_counter() {
        let mut nba = Buchi::new();
        let s1 = State::new("s1".into());
        let s2 = State::new("s2".into());
        let a = Word::new("a".into());
        let b = Word::new("b".into());

        nba.add_transition(&s1, &s2, &a);
        nba.add_transition(&s2, &s1, &b);

        nba.initial_state(&s1);
        nba.accepting_state(&s2);

        let result = nba.verify();
        assert!(result.is_err(), "{:?}", result);
        let trace = result.unwrap_err();
        assert!(
            format!("{}", trace) == String::from("(a)(b,a)ω"),
            "{}",
            trace
        )
    }

    #[test]
    pub fn verify_empty() {
        let mut nba = Buchi::new();
        let s1 = State::new("s1".into());
        let s2 = State::new("s2".into());
        let a = Word::new("a".into());
        let b = Word::new("b".into());

        nba.add_transition(&s1, &s2, &a);
        nba.add_transition(&s2, &s1, &b);

        nba.initial_state(&s1);

        let result = nba.verify();
        assert!(result.is_ok(), "{:?}", result);
    }
}
