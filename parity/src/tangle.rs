use log::debug;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use itertools::{Either, Itertools};
use petgraph::graph::NodeIndex;

use crate::{Graph, Owner, Solution};

#[derive(Eq, PartialEq, Hash, Clone)]
struct Tangle {
    winner: Owner,
    vertices: BTreeSet<NodeIndex>,
    strategy: BTreeMap<NodeIndex, NodeIndex>,
}

impl Tangle {
    fn escapes(&self, graph: &Graph) -> BTreeSet<NodeIndex> {
        let mut escapes = BTreeSet::new();
        for v in self
            .vertices
            .iter()
            .filter(|v| graph.inner[**v].owner != self.winner)
        {
            escapes.extend(
                graph
                    .inner
                    .neighbors(*v)
                    .filter(|n| !self.vertices.contains(&n)),
            )
        }

        escapes
    }

    fn neighbors(&self, graph: &Graph) -> HashSet<NodeIndex> {
        let mut neighbors = HashSet::new();
        for v in &self.vertices {
            neighbors.extend(graph.inner.neighbors(*v));
        }
        neighbors
    }

    fn is_closed(&self, graph: &Graph) -> bool {
        let (z_alpha, z_beta): (Vec<NodeIndex>, Vec<NodeIndex>) = self
            .vertices
            .iter()
            .partition(|v| graph.inner[**v].owner == self.winner);

        // Trivial case of single edge
        if z_alpha.len() == 1
            && self.strategy.is_empty()
            && graph.inner.neighbors(z_alpha[0]).count() == 0
        {
            return true;
        }

        for v in z_alpha {
            let neighbors = graph.inner.neighbors(v).collect_vec();
            if neighbors.is_empty() {
                continue;
            }
            if !neighbors.into_iter().any(|n| self.vertices.contains(&n)) {
                return false;
            }
        }

        for v in z_beta {
            if graph
                .inner
                .neighbors(v)
                .any(|n| !self.vertices.contains(&n))
            {
                return false;
            }
        }

        true
    }

    fn debug(&self, graph: &Graph) -> String {
        format!(
            "{} owns {} with strat ({})",
            self.winner,
            graph.debug(&self.vertices),
            self.strategy
                .iter()
                .map(|(k, v)| format!("{} -> {}", graph.debug_vertice(*k), graph.debug_vertice(*v)))
                .join(", ")
        )
    }
}

impl Graph {
    fn tangle_attract(
        &self,
        tangles: &HashSet<Tangle>,
        attractor: &HashSet<NodeIndex>,
        player: Owner,
        strategy: &HashMap<NodeIndex, NodeIndex>,
    ) -> Tangle {
        let mut z: BTreeSet<_> = attractor.iter().cloned().collect();
        let mut q = z.iter().cloned().collect_vec();
        let mut strategy: BTreeMap<_, _> = strategy
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        let region: HashSet<_> = self.inner.node_indices().collect();

        while let Some(v) = q.pop() {
            for u in self
                .inner
                .neighbors_directed(v, petgraph::EdgeDirection::Incoming)
            {
                if region.contains(&u)
                    && !z.contains(&u)
                    && (self.player_vertices(player).contains(&u)
                        || self.inner.neighbors(u).all(|v| z.contains(&v)))
                {
                    z.insert(u);
                    if !q.contains(&u) {
                        q.push(u);
                    }
                }

                if z.intersection(&self.player_vertices(player).collect::<BTreeSet<_>>())
                    .contains(&u)
                    && !strategy.contains_key(&u)
                {
                    strategy.insert(u, v);
                }
            }

            // Check adjacent tangles also owned by player player_alpha
            // If they are check if all escape options for player beta lead to the current tangle
            for tangle in tangles
                .into_iter()
                .filter(|t| t.winner == player && t.neighbors(self).contains(&v))
            {
                if tangle.vertices.is_subset(&z) {
                    continue;
                }
                if tangle
                    .vertices
                    .iter()
                    .all(|v| region.contains(&v) || z.contains(&v))
                    && tangle.escapes(self).is_subset(&z)
                {
                    let mut u_prime = tangle.vertices.clone();
                    u_prime.retain(|v| !z.contains(&v));
                    z.extend(&tangle.vertices);
                    // Extending queue with all the vertices
                    for v in &tangle.vertices {
                        if !q.contains(v) {
                            q.push(*v);
                        }
                    }
                    strategy.extend(tangle.strategy.iter().filter(|(k, _)| u_prime.contains(&k)));
                }
            }
        }

        debug!(
            "{} attracted {} in {}",
            self.debug(attractor),
            self.debug(&z),
            self.debug_all()
        );

        Tangle {
            vertices: z,
            strategy,
            winner: player,
        }
    }

    // Find new tangles in G given existing tangles
    fn search(&self, tangles: &HashSet<Tangle>) -> HashSet<Tangle> {
        if self.inner.node_count() == 0 {
            return HashSet::new();
        }

        let p = self.highest_priority().unwrap();
        let player_alpha = Owner::from_usize(p);
        let highest_priority_vertices = self
            .inner
            .node_indices()
            .filter(|v| self.inner[*v].priority == p)
            .collect();
        let t = self.tangle_attract(
            &tangles,
            &highest_priority_vertices,
            player_alpha,
            &HashMap::new(),
        );

        if t.is_closed(self) {
            debug!(
                "new closed tangle added {} in {}",
                t.debug(self),
                self.debug_all()
            );
            let mut recursive_result = self.remove_vertices_b_tree(&t.vertices).search(tangles);
            recursive_result.insert(t);
            recursive_result
        } else {
            debug!(
                "tangle t {} was open in {}",
                t.debug(self),
                self.debug_all()
            );
            self.remove_vertices_b_tree(&t.vertices).search(tangles)
        }
    }

    pub fn tangle(&self) -> Solution {
        let mut w_even = HashSet::new();
        let mut sigma_even = HashMap::new();
        let mut w_odd = HashSet::new();
        let mut sigma_odd = HashMap::new();
        let mut tangles: HashSet<Tangle> = HashSet::new();

        let mut g = self.clone();

        while g.inner.node_count() != 0 {
            debug!("searching for new tangles in g: {}", g.debug_all());
            debug!(
                "current tangles: {}",
                tangles
                    .iter()
                    .map(|t| format!("{}", self.debug(&t.vertices)))
                    .join(", ")
            );
            let y = g.search(&tangles);
            debug!(
                "found new tangles: {}",
                y.iter()
                    .map(|t| format!("{}", self.debug(&t.vertices)))
                    .join(", ")
            );
            tangles.extend(y.iter().filter(|t| !t.escapes(&g).is_empty()).cloned());
            let d: HashSet<_> = y
                .iter()
                .filter(|t| t.escapes(&g).is_empty())
                .cloned()
                .collect();

            debug!(
                "new dominions: {}",
                d.iter().map(|t| t.debug(&g)).join(", ")
            );

            if !d.is_empty() {
                // Split D into even and odd
                let (d_even, d_odd): (Vec<_>, Vec<_>) =
                    d.iter().partition_map(|t| match t.winner {
                        Owner::Even => Either::Left(t.vertices.clone()),
                        Owner::Odd => Either::Right(t.vertices.clone()),
                    });
                let (d_even, d_odd): (HashSet<_>, HashSet<_>) = (
                    d_even.into_iter().flatten().collect(),
                    d_odd.into_iter().flatten().collect(),
                );

                let (d_even_strat, d_odd_strat): (Vec<_>, Vec<_>) =
                    d.iter().partition_map(|t| match t.winner {
                        Owner::Even => Either::Left(t.strategy.clone()),
                        Owner::Odd => Either::Right(t.strategy.clone()),
                    });
                let (d_even_strat, d_odd_strat): (HashMap<_, _>, HashMap<_, _>) = (
                    d_even_strat.into_iter().flatten().collect(),
                    d_odd_strat.into_iter().flatten().collect(),
                );

                let d_plus_even = g.tangle_attract(&tangles, &d_even, Owner::Even, &d_even_strat);
                let d_plus_odd = g.tangle_attract(&tangles, &d_odd, Owner::Odd, &d_odd_strat);
                debug!("Adding {} to w_even", self.debug(&d_plus_even.vertices));
                debug!("Adding {} to w_odd", self.debug(&d_plus_odd.vertices));

                g = g.remove_vertices_b_tree(&d_plus_even.vertices);
                g = g.remove_vertices_b_tree(&d_plus_odd.vertices);

                w_even.extend(d_plus_even.vertices);
                sigma_even.extend(d_plus_even.strategy);
                w_odd.extend(d_plus_odd.vertices);
                sigma_odd.extend(d_plus_odd.strategy);

                // Clean up tangles
                tangles = tangles
                    .into_iter()
                    .filter(|t| {
                        t.vertices
                            .iter()
                            .all(|v| g.inner.node_indices().contains(&v))
                    })
                    .collect();
            }
        }

        // Construct solution
        self.construct_solution(w_even, w_odd, sigma_even, sigma_odd)
    }
}
