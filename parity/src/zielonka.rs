use std::collections::{HashMap, HashSet};

use itertools::Itertools;
use petgraph::stable_graph::NodeIndex;

use crate::{Graph, Owner, Solution, Strategy};

impl Graph {
    fn attract(
        &self,
        z: &HashSet<NodeIndex>,
        player: Owner,
        strategy: &HashMap<NodeIndex, NodeIndex>,
    ) -> (HashSet<NodeIndex>, HashMap<NodeIndex, NodeIndex>) {
        let mut z = z.clone();
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

        (z, strategy)
    }

    pub fn zielonka(&self) -> Solution {
        if self.inner.node_count() == 0 {
            return Solution::empty();
        }

        let (w_0, w_1, s_0, s_1) = self.zielonka_r();

        let mut strat = s_0;
        strat.extend(s_1.into_iter());
        let mut strategy = strat
            .into_iter()
            .map(|(k, v)| {
                let id = self.inner[k].id;
                let target_id = self.inner[v].id;
                let winner = if w_0.contains(&k) {
                    Owner::Even
                } else {
                    Owner::Odd
                };
                let s = Strategy {
                    winner,
                    next_node_id: Some(target_id),
                };
                (id, s)
            })
            .collect::<HashMap<_, _>>();

        for v in self.inner.node_indices() {
            let id = self.inner[v].id;
            if !strategy.contains_key(&id) {
                let winner = if w_0.contains(&v) {
                    Owner::Even
                } else {
                    Owner::Odd
                };
                let s = Strategy {
                    winner,
                    next_node_id: None,
                };
                strategy.insert(id, s);
            }
        }

        let w_0 = w_0
            .into_iter()
            .map(|w| &self.inner[w])
            .collect::<HashSet<_>>();
        let w_1 = w_1
            .into_iter()
            .map(|w| &self.inner[w])
            .collect::<HashSet<_>>();

        Solution {
            even_region: w_0,
            odd_region: w_1,
            strategy,
        }
    }

    fn remove_vertices(&self, purge: &HashSet<NodeIndex>) -> Self {
        Graph {
            inner: self.inner.filter_map(
                |v, w| {
                    if purge.contains(&v) {
                        None
                    } else {
                        Some(w.clone())
                    }
                },
                |_, _| Some(()),
            ),
        }
    }

    fn zielonka_r(
        &self,
    ) -> (
        HashSet<NodeIndex>,
        HashSet<NodeIndex>,
        HashMap<NodeIndex, NodeIndex>,
        HashMap<NodeIndex, NodeIndex>,
    ) {
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
            let w_alpha = match player_alpha {
                Owner::Even => &mut w_even,
                Owner::Odd => &mut w_odd,
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
            let (mut w_even, mut w_odd, mut strat_even, mut strat_odd) =
                self.remove_vertices(&b).zielonka_r();
            let strat_beta = match player_beta {
                Owner::Even => {
                    w_even.extend(b);
                    &mut strat_even
                }
                Owner::Odd => {
                    w_odd.extend(b);
                    &mut strat_odd
                }
            };
            strat_beta.extend(strat_b);
            (w_even, w_odd, strat_even, strat_odd)
        }
    }
}
