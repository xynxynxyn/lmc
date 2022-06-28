use crate::{Graph, Owner, Solution, Strategy};
use itertools::Itertools;
use petgraph::graph::NodeIndex;
use std::collections::{BTreeSet, HashMap, HashSet};

impl Graph {
    fn highest_priority(&self) -> Option<usize> {
        self.inner.node_weights().map(|n| n.priority).max()
    }

    fn winner(&self, v: NodeIndex, z: &BTreeSet<NodeIndex>) -> usize {
        let p = self
            .inner
            .node_weight(v)
            .expect("Could not find node with given weight")
            .priority
            % 2;
        if !z.contains(&v) {
            p
        } else {
            1 - p
        }
    }
    fn onestep(&self, v: NodeIndex, z: &BTreeSet<NodeIndex>) -> (usize, Option<NodeIndex>) {
        let p = self
            .inner
            .node_weight(v)
            .expect("Could not find node with given weight");

        match p.owner {
            Owner::Even => {
                match self.inner.neighbors(v).into_iter().find_map(|n| {
                    if self.winner(n, z) == 0 {
                        Some((0, Some(n)))
                    } else {
                        None
                    }
                }) {
                    Some(e) => e,
                    None => (1, None),
                }
            }
            Owner::Odd => {
                match self.inner.neighbors(v).into_iter().find_map(|n| {
                    if self.winner(n, z) == 1 {
                        Some((1, Some(n)))
                    } else {
                        None
                    }
                }) {
                    Some(e) => e,
                    None => (0, None),
                }
            }
        }
    }

    pub fn fpi<'a>(&'a self) -> Solution<'a> {
        let mut z = BTreeSet::new();
        let mut frozen = HashMap::new();
        let mut strategy = HashMap::new();
        let mut p = 0;
        let max_priority = self
            .highest_priority()
            .expect("Graph was empty, cannot determine highest priority");

        while p <= max_priority {
            let parity = p % 2;
            let y: BTreeSet<_> = self
                .inner
                .node_indices()
                .into_iter()
                .filter(|v| *&self.inner[*v].priority == p) // All vertices with priority p
                .filter(|v| !frozen.contains_key(v) && !z.contains(v)) // Only if the vertex is not frozen and not in Z
                .collect();

            let mut chg = false;
            for v in y {
                let (alpha, strat) = self.onestep(v, &z);
                strategy.insert(v, strat);
                if alpha != parity {
                    chg = true;
                    z.insert(v);
                }
            }

            if chg {
                for v in self
                    .inner
                    .node_indices()
                    .into_iter()
                    .filter(|v| *&self.inner[*v].priority < p)
                    .filter(|v| !frozen.contains_key(v))
                    .collect_vec()
                {
                    if self.winner(v, &z) == (p + 1) % 2 {
                        frozen.insert(v, p);
                    } else {
                        z.remove(&v);
                    }
                }
                p = 0;
            } else {
                for v in self
                    .inner
                    .node_indices()
                    .into_iter()
                    .filter(|v| *&self.inner[*v].priority < p)
                    .filter(|v| frozen.get(v) == Some(&p))
                    .collect_vec()
                {
                    frozen.remove(&v);
                }
                p += 1;
            }
        }
        let (w_0, w_1): (BTreeSet<_>, BTreeSet<_>) = self
            .inner
            .node_indices()
            .into_iter()
            .partition(|v| self.winner(*v, &z) == 0);

        let s_0 = w_0
            .iter()
            .filter(|v| *&self.inner[**v].owner == Owner::Even && strategy.contains_key(*v))
            .map(|v| (&self.inner[*v], strategy[v]));
        let s_1 = w_1
            .iter()
            .filter(|v| *&self.inner[**v].owner == Owner::Odd && strategy.contains_key(*v))
            .map(|v| (&self.inner[*v], strategy[v]));

        let mut strategy = HashMap::new();
        for (v, t) in s_0 {
            strategy.insert(
                v.id,
                Strategy {
                    winner: v.owner,
                    next_node_id: t.map(|a| self.inner[a].id),
                },
            );
        }
        for (v, t) in s_1 {
            strategy.insert(
                v.id,
                Strategy {
                    winner: v.owner,
                    next_node_id: t.map(|a| self.inner[a].id),
                },
            );
        }

        let w_0: HashSet<_> = w_0.into_iter().map(|v| &self.inner[v]).collect();
        let w_1: HashSet<_> = w_1.into_iter().map(|v| &self.inner[v]).collect();

        for w in &w_0 {
            if !strategy.contains_key(&w.id) {
                strategy.insert(
                    w.id,
                    Strategy {
                        winner: Owner::Even,
                        next_node_id: None,
                    },
                );
            }
        }

        for w in &w_1 {
            if !strategy.contains_key(&w.id) {
                strategy.insert(
                    w.id,
                    Strategy {
                        winner: Owner::Odd,
                        next_node_id: None,
                    },
                );
            }
        }

        Solution {
            even_region: w_0,
            odd_region: w_1,
            strategy,
        }
    }
}
