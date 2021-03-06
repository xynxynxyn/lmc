use crate::{Graph, Owner, Solution};
use colored::Colorize;
use itertools::Itertools;
use petgraph::graph::NodeIndex;
use std::collections::{BTreeSet, HashMap, HashSet};

impl Graph {
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
        log::info!("solving with FPI + freezing");
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
                if let Some(s) = strat {
                    strategy.insert(v, s);
                }
                if alpha != parity {
                    chg = true;
                    log::debug!("distractions <- {}", self.debug_vertice(v));
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
                        log::debug!(
                            "{} {} at priority {}",
                            "freezing".cyan(),
                            self.debug_vertice(v),
                            p
                        );
                        frozen.insert(v, p);
                    } else {
                        log::debug!("{} {}", "resetting".red(), self.debug_vertice(v));
                        z.remove(&v);
                    }
                }
                p = 0;
                log::debug!("restarting after finding distractions");
            } else {
                for v in self
                    .inner
                    .node_indices()
                    .into_iter()
                    .filter(|v| *&self.inner[*v].priority < p)
                    .filter(|v| frozen.get(v) == Some(&p))
                    .collect_vec()
                {
                    log::debug!("{} {}", "thawing".bright_red(), self.debug_vertice(v),);
                    frozen.remove(&v);
                }
                p += 1;
            }
        }

        let (w_0, w_1): (HashSet<_>, HashSet<_>) = self
            .inner
            .node_indices()
            .into_iter()
            .partition(|v| self.winner(*v, &z) == 0);

        let (s_0, s_1) = strategy.into_iter().partition(|(k, _)| w_0.contains(&k));

        self.construct_solution(w_0, w_1, s_0, s_1)
    }
}
