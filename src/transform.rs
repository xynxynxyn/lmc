// Transform an LTL formula to a GNBA/NBA

use std::collections::{BTreeSet, HashMap};

use buchi::nba::Buchi;
use ltl::transform::{Expr, Formula};

pub fn ltl_to_gnba(formula: &Formula) -> Buchi {
    let mut gnba = Buchi::new();
    let mut states = HashMap::new();
    let formula = formula.pnf();
    let closure = formula.closure();
    let elementary = formula.elementary();
    let alphabet = formula.alphabet();

    // Populate the states
    for e in &elementary {
        states.insert(e, gnba.new_state());
    }

    // Set initial states
    for (b_set, state) in &states {
        if b_set.contains(&formula.root_expr) {
            gnba.set_initial_state(*state);
        }
    }

    // Set accepting states
    for expr in &closure {
        if let e @ Expr::Until(_, rhs) = expr {
            for (b_set, state) in &states {
                if !b_set.contains(e) || b_set.contains(rhs) {
                    gnba.set_accepting_state(*state);
                }
            }
        }
    }

    // Configure transitions
    for b in &elementary {
        let intersection = BTreeSet::from_iter(b.intersection(&alphabet).map(Expr::clone));
        if intersection.is_empty() {
            // This state has no transitions
            continue;
        }

        let label = Expr::print_set(&intersection);

        for expr in &closure {
            if let Expr::Next(next) = expr {
                for target in elementary.iter().filter(|e| e.contains(next)) {
                    gnba.add_transition(
                        *states.get(b).unwrap(),
                        *states.get(target).unwrap(),
                        label.clone(),
                    );
                }
            } else if let until @ Expr::Until(lhs, rhs) = expr {
                if b.contains(rhs) {
                    // Connect this state to every other state
                    for target in &elementary {
                        gnba.add_transition(
                            *states.get(b).unwrap(),
                            *states.get(target).unwrap(),
                            label.clone(),
                        )
                    }
                } else if b.contains(lhs) {
                    for target in elementary.iter().filter(|e| e.contains(until)) {
                        gnba.add_transition(
                            *states.get(b).unwrap(),
                            *states.get(target).unwrap(),
                            label.clone(),
                        )
                    }
                }
            }
        }
    }

    println!("Mapping:");
    for (k, v) in states {
        println!("{:?}: {}", v, Expr::print_set(k));
    }
    gnba
}
