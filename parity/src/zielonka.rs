use std::collections::{HashMap, HashSet};

use itertools::Itertools;
use petgraph::stable_graph::NodeIndex;

use crate::{Graph, Owner, Solution};

impl Graph {
    fn attract(&self, z: &HashSet<NodeIndex>, player: Owner) -> HashSet<NodeIndex> {
        let mut z = z.clone();
        let mut q: Vec<_> = z.iter().cloned().collect();

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
            }
        }

        z
    }

    pub fn zielonka(&self) -> Solution {
        if self.inner.node_count() == 0 {
            return Solution::empty();
        }

        let (w_0, w_1) = self.zielonka_r();
        //let mut strat: HashMap<NodeIndex, NodeIndex> = HashMap::new();
        //let mut strategy = strat
        //    .into_iter()
        //    .map(|(k, v)| {
        //        let id = self.inner[k].id;
        //        let target_id = self.inner[v].id;
        //        let winner = if w_0.contains(&k) {
        //            Owner::Even
        //        } else {
        //            Owner::Odd
        //        };
        //        let s = Strategy {
        //            winner,
        //            next_node_id: Some(target_id),
        //        };
        //        (id, s)
        //    })
        //    .collect::<HashMap<_, _>>();

        //for v in self.inner.node_indices() {
        //    let id = self.inner[v].id;
        //    if !strategy.contains_key(&id) {
        //        let winner = if w_0.contains(&v) {
        //            Owner::Even
        //        } else {
        //            Owner::Odd
        //        };
        //        let s = Strategy {
        //            winner,
        //            next_node_id: None,
        //        };
        //        strategy.insert(id, s);
        //    }
        //}

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
            strategy: HashMap::new(),
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

    fn zielonka_r(&self) -> (HashSet<NodeIndex>, HashSet<NodeIndex>) {
        if self.inner.node_count() == 0 {
            return (HashSet::new(), HashSet::new());
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
        let a = self.attract(&z, player_alpha);

        // Recursively find out the winning areas in that subgraph
        let (mut w_even, mut w_odd) = self.remove_vertices(&a).zielonka_r();

        let w_beta = match player_beta {
            Owner::Even => &w_even,
            Owner::Odd => &w_odd,
        };

        let b = self.attract(w_beta, player_beta);

        if b == *w_beta {
            match player_alpha {
                Owner::Even => w_even.extend(a),
                Owner::Odd => w_odd.extend(a),
            }
            (w_even, w_odd)
        } else {
            let (mut w_even, mut w_odd) = self.remove_vertices(&b).zielonka_r();
            match player_beta {
                Owner::Even => w_even.extend(b),
                Owner::Odd => w_odd.extend(b),
            }
            (w_even, w_odd)
        }
    }
}
