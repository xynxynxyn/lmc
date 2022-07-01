use std::collections::{HashMap, HashSet};

use colored::Colorize;
use itertools::Itertools;
use petgraph::stable_graph::NodeIndex;

use crate::{Graph, Owner, Solution};

impl Graph {
    fn attract(
        &self,
        attractor: &HashSet<NodeIndex>,
        player: Owner,
        strategy: &HashMap<NodeIndex, NodeIndex>,
    ) -> (HashSet<NodeIndex>, HashMap<NodeIndex, NodeIndex>) {
        let mut z = attractor.clone();
        let mut q: Vec<_> = z.iter().cloned().collect();
        let mut strategy = strategy.clone();

        while let Some(v) = q.pop() {
            for u in self
                .inner
                .neighbors_directed(v, petgraph::EdgeDirection::Incoming)
            {
                if !z.contains(&u)
                    && (self.player_vertices(player).contains(&u)
                        || self.inner.neighbors(u).all(|v| z.contains(&v)))
                {
                    z.insert(u);
                    q.push(u);
                }

                if z.intersection(&self.player_vertices(player).collect::<HashSet<_>>())
                    .contains(&u)
                    && !strategy.contains_key(&u)
                {
                    strategy.insert(u, v);
                }
            }
        }

        log::debug!(
            "{} {} {} in subgraph {}",
            self.debug(attractor),
            "attracts".green(),
            self.debug(&z),
            self.debug_all()
        );
        (z, strategy)
    }

    pub fn zielonka(&self) -> Solution {
        log::info!("solving with zielonka's");
        if self.inner.node_count() == 0 {
            return Solution::empty();
        }

        let (w_0, w_1, s_0, s_1) = self.zielonka_r();

        self.construct_solution(w_0, w_1, s_0, s_1)
    }

    fn zielonka_r(
        &self,
    ) -> (
        HashSet<NodeIndex>,
        HashSet<NodeIndex>,
        HashMap<NodeIndex, NodeIndex>,
        HashMap<NodeIndex, NodeIndex>,
    ) {
        log::debug!("applying zielonka's to graph {}", self.debug_all());
        if self.inner.node_count() == 0 {
            return (
                HashSet::new(),
                HashSet::new(),
                HashMap::new(),
                HashMap::new(),
            );
        }

        let highest_priority = self.highest_priority().unwrap();
        let player_alpha = Owner::from_usize(highest_priority);
        let player_beta = player_alpha.neg();

        // Collect the vertices of highest priority for initial attractor
        let z = self
            .inner
            .node_indices()
            .filter(|v| self.inner[*v].priority == highest_priority)
            .collect::<HashSet<_>>();

        // Calculate the attractor for the highest priority vertices
        let (a, strat_a) = self.attract(&z, player_alpha, &HashMap::new());

        // Recursively find out the winning areas in that subgraph
        let (mut w_even, mut w_odd, mut strat_even, mut strat_odd) =
            self.remove_vertices(&a).zielonka_r();

        let (strat_alpha, w_beta, strat_beta) = match player_alpha {
            Owner::Even => (&mut strat_even, &w_odd, &strat_odd),
            Owner::Odd => (&mut strat_odd, &w_even, &strat_even),
        };

        let (b, strat_b) = self.attract(w_beta, player_beta, strat_beta);

        if b == *w_beta {
            log::debug!(
                "{}({}) {} {}",
                "α".blue(),
                player_alpha,
                "wins".blue(),
                self.debug(&a),
            );
            let w_alpha = match player_alpha {
                Owner::Even => {
                    log::debug!("extending {} by {}", "W_even".blue(), self.debug(&a));
                    &mut w_even
                }
                Owner::Odd => {
                    log::debug!("extending {} by {}", "W_odd".red(), self.debug(&a));
                    &mut w_odd
                }
            };
            w_alpha.extend(a);
            strat_alpha.extend(strat_a);
            for v in z {
                if !strat_alpha.contains_key(&v) {
                    let arbitrary_target = self
                        .inner
                        .neighbors(v)
                        .filter(|v| w_alpha.contains(&v))
                        .next();
                    if let Some(t) = arbitrary_target {
                        strat_alpha.insert(v, t);
                    }
                }
            }

            (w_even, w_odd, strat_even, strat_odd)
        } else {
            log::debug!(
                "{}({}) {} {}",
                "β".red(),
                player_beta,
                "wins".red(),
                self.debug(&b),
            );
            let (mut w_even, mut w_odd, mut strat_even, mut strat_odd) =
                self.remove_vertices(&b).zielonka_r();
            log::debug!(
                "{} {} and {} with {} and {}",
                "overwrote".magenta(),
                "W_even".blue(),
                "W_odd".red(),
                self.debug(&w_even),
                self.debug(&w_odd)
            );
            let strat_beta = match player_beta {
                Owner::Even => {
                    log::debug!("extending {} by {}", "W_even".blue(), self.debug(&b));
                    w_even.extend(b);
                    &mut strat_even
                }
                Owner::Odd => {
                    log::debug!("extending {} by {}", "W_odd".red(), self.debug(&b));
                    w_odd.extend(b);
                    &mut strat_odd
                }
            };
            strat_beta.extend(strat_b);
            (w_even, w_odd, strat_even, strat_odd)
        }
    }
}
