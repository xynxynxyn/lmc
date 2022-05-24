// Transform an LTL formula to a GNBA/NBA

use std::collections::{BTreeSet, HashMap};

use buchi::nba::Buchi;
use ltl::formula::{Expr, Formula};

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

        let label = Expr::print_set(&intersection);

        for expr in &closure {
            if let next @ Expr::Next(ex) = expr {
                if b.contains(next) {
                    for target in elementary.iter().filter(|e| e.contains(ex)) {
                        gnba.add_transition(
                            *states.get(b).unwrap(),
                            *states.get(target).unwrap(),
                            label.clone(),
                        );
                    }
                } else {
                    for target in elementary.iter().filter(|e| !e.contains(ex)) {
                        gnba.add_transition(
                            *states.get(b).unwrap(),
                            *states.get(target).unwrap(),
                            label.clone(),
                        );
                    }
                }
            } else if let until @ Expr::Until(lhs, rhs) = expr {
                if b.contains(until) {
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
                } else {
                    for target in elementary
                        .iter()
                        .filter(|e| !b.contains(rhs) && (!b.contains(lhs) || e.contains(until)))
                    {
                        gnba.add_transition(
                            *states.get(b).unwrap(),
                            *states.get(target).unwrap(),
                            label.clone(),
                        )
                    }
                }
            } else if let release @ Expr::Release(lhs, rhs) = expr {
                if b.contains(release) {
                    // The condition is fulfilled, this state can transition to any other
                    if b.contains(lhs) && b.contains(rhs) {
                        for target in &elementary {
                            gnba.add_transition(
                                *states.get(b).unwrap(),
                                *states.get(target).unwrap(),
                                label.clone(),
                            )
                        }
                    // If only the right side is true then all states that also contain this proposition are potential next states
                    } else if b.contains(rhs) {
                        for target in elementary.iter().filter(|e| e.contains(release)) {
                            gnba.add_transition(
                                *states.get(b).unwrap(),
                                *states.get(target).unwrap(),
                                label.clone(),
                            )
                        }
                    }
                // If the current state does not contain the release proposition to the opposite
                } else {
                    for target in elementary.iter().filter(|e| {
                        !(b.contains(lhs) && b.contains(rhs))
                            && (b.contains(rhs) || e.contains(release))
                    }) {
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

    gnba
}
