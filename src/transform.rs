// Transform an LTL formula to a GNBA/NBA

use std::collections::{BTreeSet, HashMap, HashSet};

use buchi::nba::Buchi;
use ltl::{Expr, Formula};

pub fn ltl_to_gnba(formula: &Formula) -> Buchi {
    let mut gnba = Buchi::new();
    let mut states = HashMap::new();
    let formula = formula.pnf();
    let closure = formula.closure();
    let elementary = formula.elementary();
    let alphabet = formula.alphabet();

    // Populate the states
    for e in &elementary {
        states.insert(e, gnba.new_labeled_state(Expr::print_set(e)));
    }

    // Set initial states
    for (b_set, state) in &states {
        if b_set.contains(&formula.root_expr) {
            gnba.set_initial_state(*state);
        }
    }

    // Set accepting states
    // TODO this should generate a set of sets of states
    // Then also change the verification procedure
    // This should be simply just checking that all states in one acceptance set are contained within a single SCC
    for expr in &closure {
        if let until @ Expr::Until(_, rhs) = expr {
            let accepting_set = states
                .iter()
                .filter_map(|(b_set, state)| {
                    if !b_set.contains(until) || b_set.contains(rhs) {
                        Some(state)
                    } else {
                        None
                    }
                })
                .cloned()
                .collect::<HashSet<_>>();
            gnba.add_accepting_set(accepting_set.into_iter());
        }
    }

    // Configure transitions
    for s in &elementary {
        let intersection = BTreeSet::from_iter(s.intersection(&alphabet).cloned());

        let label = Expr::print_set(&intersection);

        let mut target_sets = Vec::<BTreeSet<&BTreeSet<Expr>>>::new();
        for expr in &closure {
            let potential_targets = if let next @ Expr::Next(ex) = expr {
                elementary
                    .iter()
                    .filter(|s_prime| {
                        (s.contains(next) && s_prime.contains(ex))
                            || (!s.contains(next) && !s_prime.contains(ex))
                    })
                    .collect()
            } else if let until @ Expr::Until(a, b) = expr {
                if s.contains(until) {
                    elementary
                        .iter()
                        .filter(|s_prime| {
                            s.contains(b) || (s.contains(a) && s_prime.contains(until))
                        })
                        .collect()
                } else {
                    elementary
                        .iter()
                        .filter(|s_prime| {
                            !(s.contains(b) || (s.contains(a) && s_prime.contains(until)))
                        })
                        .collect()
                }
            } else if let release @ Expr::Release(a, b) = expr {
                if s.contains(release) {
                    elementary
                        .iter()
                        .filter(|s_prime| {
                            (s.contains(a) && s.contains(b))
                                || (s.contains(b) && s_prime.contains(release))
                        })
                        .collect()
                // If the current state does not contain the release proposition to the opposite
                } else {
                    elementary
                        .iter()
                        .filter(|s_prime| {
                            !((s.contains(a) && s.contains(b))
                                || (s.contains(b) && s_prime.contains(release)))
                        })
                        .collect()
                }
            } else {
                continue;
            };

            target_sets.push(potential_targets);
        }

        let mut all_states: BTreeSet<_> = elementary.iter().collect();
        for t in &target_sets {
            all_states = all_states.intersection(t).cloned().collect();
        }

        let intersection = all_states;

        // Add the states
        for t in intersection {
            gnba.add_transition(
                *states.get(s).unwrap(),
                *states.get(t).unwrap(),
                label.clone(),
            );
        }
    }

    gnba
}
