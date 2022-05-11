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
        let s1 = State::new("s1".into());
        let s2 = State::new("s2".into());
        let s3 = State::new("s3".into());
        let s4 = State::new("s4".into());
        let s5 = State::new("s5".into());
        let s6 = State::new("s6".into());
        let s7 = State::new("s7".into());
        let s8 = State::new("s8".into());
        let w = Word::new("w".into());

        nba.add_transition(&s1, &s2, &w);
        nba.add_transition(&s2, &s3, &w);
        nba.add_transition(&s3, &s1, &w);
        nba.add_transition(&s4, &s2, &w);
        nba.add_transition(&s4, &s3, &w);
        nba.add_transition(&s5, &s4, &w);
        nba.add_transition(&s4, &s5, &w);
        nba.add_transition(&s6, &s3, &w);
        nba.add_transition(&s5, &s6, &w);
        nba.add_transition(&s6, &s7, &w);
        nba.add_transition(&s7, &s6, &w);
        nba.add_transition(&s8, &s7, &w);
        nba.add_transition(&s8, &s6, &w);
        nba.add_transition(&s8, &s8, &w);

        let components = nba.tarjans();
        assert!(components.len() == 4);
    }
}
