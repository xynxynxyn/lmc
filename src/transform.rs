// Transform an LTL formula to a GNBA/NBA

use std::collections::{BTreeSet, HashMap, HashSet, VecDeque};

use buchi::nba::Buchi;
use itertools::Itertools;
use ltl::{Expr, Formula};
use petri::PetriNet;

pub fn _ts_and_buchi_product(ts: Buchi, a: Buchi) -> Buchi {
    let mut product = Buchi::new();
    let mut states = HashMap::new();
    for ts_transitions in ts.transitions() {
        for a_transitions in a.transitions() {
            if a_transitions.label == ts_transitions.to {
                let source_label = format!(
                    "<s{},q{}>({},{})",
                    ts_transitions.from_state.id,
                    a_transitions.from_state.id,
                    ts_transitions.from,
                    a_transitions.from
                );
                let target_label = format!(
                    "<s{},q{}>({},{})",
                    ts_transitions.to_state.id,
                    a_transitions.to_state.id,
                    ts_transitions.to,
                    a_transitions.to
                );

                let source_state = states
                    .entry(source_label.clone())
                    .or_insert_with(|| product.new_labeled_state(source_label))
                    .clone();
                let target_state = states
                    .entry(target_label.clone())
                    .or_insert_with(|| product.new_labeled_state(target_label));

                product.add_transition(source_state, *target_state, ts_transitions.label);
            }
        }
    }

    for s0 in ts.initial_states() {
        for q_t in a
            .transitions()
            .iter()
            .filter(|t| a.initial_states().contains(&t.from_state))
        {
            if q_t.label == ts.label(s0).unwrap() {
                let init_label = format!(
                    "<s{},q{}>({},{})",
                    s0.id,
                    q_t.to_state.id,
                    ts.label(s0).unwrap(),
                    q_t.to
                );
                let init_state = states
                    .entry(init_label.clone())
                    .or_insert_with(|| product.new_labeled_state(init_label));
                product.set_initial_state(*init_state);
            }
        }
    }

    product
}

pub fn petri_to_gnba(net: PetriNet) -> Buchi {
    // Collect all markings
    let mut gnba = Buchi::new();
    let mut states = HashMap::new();

    let initial_marking = net.initial_marking();
    let initial_label = petri_state_to_string(&initial_marking.active_transitions(&net));
    let initial_state = gnba.new_labeled_state(initial_label.clone());
    states.insert(initial_label, initial_state);
    gnba.set_initial_state(initial_state);

    // Visit all markings and fill up gnba as we go
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();
    queue.push_back(initial_marking.clone());
    visited.insert(initial_marking);

    while let Some(marking) = queue.pop_front() {
        let next_markings = net
            .transitions(&marking)
            .expect("Markings are inconsistent with petri net, this shouldn't happen");
        for (label, m) in next_markings {
            // Insert transition into gnba
            let source_label = petri_state_to_string(&marking.active_transitions(&net));
            let target_label = petri_state_to_string(&m.active_transitions(&net));

            let source_state = states
                .entry(source_label.clone())
                .or_insert_with(|| gnba.new_labeled_state(source_label))
                .clone();

            let target_state = states
                .entry(target_label.clone())
                .or_insert_with(|| gnba.new_labeled_state(target_label));

            gnba.add_transition(source_state, *target_state, label);
            if !visited.contains(&m) {
                visited.insert(m.clone());
                queue.push_back(m);
            }
        }
    }

    gnba
}

fn petri_state_to_string(active_transitions: &Vec<&str>) -> String {
    format!(
        "{{{}}}",
        active_transitions
            .iter()
            .cloned()
            .sorted()
            .collect_vec()
            .join(", ")
    )
}

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

#[cfg(test)]
mod test {
    use buchi::nba::Buchi;
    use ltl::Formula;

    use super::{ltl_to_gnba, _ts_and_buchi_product};

    #[test]
    pub fn small_product() {
        let mut ts = Buchi::new();
        let s0 = ts.new_labeled_state("{a, b}".into());
        let s1 = ts.new_labeled_state("{a}".into());
        let s2 = ts.new_labeled_state("{a}".into());
        let s3 = ts.new_labeled_state("{a, b}".into());
        ts.add_transition(s0, s1, "");
        ts.add_transition(s1, s3, "");
        ts.add_transition(s3, s1, "");
        ts.add_transition(s3, s2, "");
        ts.add_transition(s2, s1, "");
        ts.add_transition(s2, s0, "");
        ts.set_initial_state(s0);

        let mut a = Buchi::new();
        let q0 = a.new_labeled_state("q0".into());
        let q1 = a.new_labeled_state("q1".into());
        let q2 = a.new_labeled_state("q2".into());
        a.add_transition(q0, q0, "{}");
        a.add_transition(q0, q0, "{b}");
        a.add_transition(q0, q1, "{a}");
        a.add_transition(q0, q1, "{a, b}");
        a.add_transition(q1, q1, "{}");
        a.add_transition(q1, q1, "{b}");
        a.add_transition(q1, q0, "{a}");
        a.add_transition(q1, q0, "{a, b}");
        a.add_transition(q1, q2, "{b}");
        a.add_transition(q1, q2, "{a, b}");
        a.add_transition(q2, q2, "{true}");

        a.set_initial_state(q0);
        a.add_accepting_set([q2]);

        println!("TS\n{}", ts.to_dot());
        println!("A\n{}", a.to_dot());
        let product = _ts_and_buchi_product(ts, a);
        println!("Product:\n{}", product.to_dot());
        panic!("Hey")
    }
}
