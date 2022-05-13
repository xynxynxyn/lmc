pub mod nba;

#[cfg(test)]
mod test {
    use super::nba::*;
    #[test]
    pub fn two_state_nba() {
        let mut nba = Buchi::new();
        let s1 = nba.new_state();
        let s2 = nba.new_state();
        let w = Word::from("w");

        nba.add_transition(s1, s2, w.clone());
        nba.add_transition(s2, s1, w.clone());

        println!("{}", nba);
        assert!(nba.transitions(s1).unwrap().get(&w).unwrap().contains(&s2));
        assert!(nba.transitions(s2).unwrap().get(&w).unwrap().contains(&s1));
    }

    #[test]
    pub fn three_state_nba() {
        let mut nba = Buchi::new();
        let s1 = nba.new_state();
        let s2 = nba.new_state();
        let s3 = nba.new_state();
        let a = Word::from("a");
        let b = Word::from("b");

        nba.add_transition(s1, s2, "a");
        nba.add_transition(s1, s3, "b");
        nba.add_transition(s3, s2, "b");

        let s1_trans = nba.transitions(s1).unwrap();
        let s2_trans = nba.transitions(s2).unwrap();
        let s3_trans = nba.transitions(s3).unwrap();

        assert!(s1_trans.get(&a).unwrap().contains(&s2));
        assert!(s1_trans.get(&b).unwrap().contains(&s3));
        assert!(s2_trans.is_empty());
        assert!(s3_trans.get(&b).unwrap().contains(&s2));
    }

    #[test]
    pub fn tarjan() {
        let mut nba = Buchi::new();
        let a = nba.new_state();
        let b = nba.new_state();
        let e = nba.new_state();
        let c = nba.new_state();
        let d = nba.new_state();
        let f = nba.new_state();
        let g = nba.new_state();
        let h = nba.new_state();

        nba.add_transition(a, b, "x");
        nba.add_transition(b, e, "x");
        nba.add_transition(e, a, "x");
        nba.add_transition(b, f, "x");
        nba.add_transition(b, c, "x");
        nba.add_transition(e, f, "x");
        nba.add_transition(c, d, "x");
        nba.add_transition(d, c, "x");
        nba.add_transition(d, h, "x");
        nba.add_transition(h, d, "x");
        nba.add_transition(c, g, "y");
        nba.add_transition(h, g, "z");
        nba.add_transition(g, f, "x");
        nba.add_transition(f, g, "x");

        let components = nba.tarjans_scc();
        assert!(components.len() == 3, "{:?}", components);
    }

    #[test]
    pub fn verify_complex() {
        let mut nba = Buchi::new();
        let a = nba.new_state();
        let b = nba.new_state();
        let e = nba.new_state();
        let c = nba.new_state();
        let d = nba.new_state();
        let f = nba.new_state();
        let g = nba.new_state();
        let h = nba.new_state();

        nba.add_transition(a, b, "x");
        nba.add_transition(b, e, "x");
        nba.add_transition(e, a, "x");
        nba.add_transition(b, f, "x");
        nba.add_transition(b, c, "x");
        nba.add_transition(e, f, "x");
        nba.add_transition(c, d, "x");
        nba.add_transition(d, c, "x");
        nba.add_transition(d, h, "x");
        nba.add_transition(h, d, "x");
        nba.add_transition(c, g, "x");
        nba.add_transition(h, g, "x");
        nba.add_transition(g, f, "y");
        nba.add_transition(f, g, "z");

        nba.set_initial_state(a);
        nba.set_accepting_state(f);

        let trace = nba.verify();
        assert!(trace.is_err(), "{:?}", trace);
        let trace = trace.unwrap_err();
        assert!(trace.omega_words.contains(&Word::from("y")), "{}", trace);
        assert!(trace.omega_words.contains(&Word::from("z")), "{}", trace)
    }

    #[test]
    pub fn verify_simple_counter() {
        let mut nba = Buchi::new();
        let s1 = nba.new_state();
        let s2 = nba.new_state();

        nba.add_transition(s1, s2, "a");
        nba.add_transition(s2, s1, "b");

        nba.set_initial_state(s1);
        nba.set_accepting_state(s2);

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
        let s1 = nba.new_state();
        let s2 = nba.new_state();

        nba.add_transition(s1, s2, "a");
        nba.add_transition(s2, s1, "b");

        nba.set_initial_state(s1);

        let result = nba.verify();
        assert!(result.is_ok(), "{:?}", result);
    }

    #[test]
    pub fn gnba_to_nba() {
        let mut gnba = Buchi::new();
        let a = gnba.new_state();
        let b = gnba.new_state();
        let c = gnba.new_state();

        gnba.add_transition(a, b, "x");
        gnba.add_transition(b, c, "y");
        gnba.add_transition(c, a, "z");

        gnba.set_initial_state(c);
        gnba.set_accepting_state(b);
        gnba.set_accepting_state(a);

        let nba = gnba.gnba_to_nba();
        assert!(nba.states().len() == 6, "{:?}", nba.states());
        // The gnba originally had 2 accepting states, the resulting nba should only have one
        assert!(gnba.accepting_states().len() == 2);
        assert!(
            nba.accepting_states().len() == 1,
            "{:?}",
            nba.accepting_states()
        );
        assert!(nba.verify().is_err(), "{}", nba);
    }
}
