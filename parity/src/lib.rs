mod fpi;
mod parse;
mod tangle;
mod zielonka;
use itertools::Itertools;
pub use parse::parse_game;
use petgraph::graph::NodeIndex;
use petgraph::stable_graph::StableDiGraph;
use std::collections::{BTreeSet, HashMap, HashSet};
use std::fmt::Display;

// The main data structure is a Graph
// Each vertex contains information:
// - What is the priority (a number from 0 to n)
// - What is the
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct MetaData {
    pub id: usize,
    pub label: Option<String>,
    pub owner: Owner,
    pub priority: usize,
}

impl MetaData {
    fn new(id: usize) -> Self {
        MetaData {
            id,
            label: None,
            owner: Owner::Even,
            priority: 0,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Owner {
    Odd,
    Even,
}

impl Owner {
    fn neg(&self) -> Self {
        match self {
            Owner::Odd => Owner::Even,
            Owner::Even => Owner::Odd,
        }
    }

    fn from_usize(u: usize) -> Self {
        if u % 2 == 0 {
            Owner::Even
        } else {
            Owner::Odd
        }
    }
}

impl Display for Owner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Owner::Odd => write!(f, "1"),
            Owner::Even => write!(f, "0"),
        }
    }
}

#[derive(Clone)]
pub struct Graph {
    inner: StableDiGraph<MetaData, ()>,
}

impl Graph {
    fn new() -> Self {
        Graph {
            inner: StableDiGraph::new(),
        }
    }

    fn highest_priority(&self) -> Option<usize> {
        self.inner.node_weights().map(|n| n.priority).max()
    }

    fn player_vertices(&self, player: Owner) -> impl Iterator<Item = NodeIndex> + '_ {
        self.inner
            .node_indices()
            .into_iter()
            .filter(move |v| self.inner[*v].owner == player)
    }

    fn remove_vertices(&self, purge: &HashSet<NodeIndex>) -> Self {
        Graph {
            inner: self.inner.filter_map(
                |v, w| {
                    if purge.contains(&&v) {
                        None
                    } else {
                        Some(w.clone())
                    }
                },
                |_, _| Some(()),
            ),
        }
    }

    fn remove_vertices_b_tree(&self, purge: &BTreeSet<NodeIndex>) -> Self {
        Graph {
            inner: self.inner.filter_map(
                |v, w| {
                    if purge.contains(&&v) {
                        None
                    } else {
                        Some(w.clone())
                    }
                },
                |_, _| Some(()),
            ),
        }
    }

    fn construct_solution(
        &self,
        w_0: HashSet<NodeIndex>,
        w_1: HashSet<NodeIndex>,
        s_0: HashMap<NodeIndex, NodeIndex>,
        s_1: HashMap<NodeIndex, NodeIndex>,
    ) -> Solution {
        log::info!("constructing solution from regions and strategies");
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

    fn debug<'a, T>(&'a self, vertices: T) -> String
    where
        T: IntoIterator<Item = &'a NodeIndex>,
    {
        format!(
            "{{{}}}",
            vertices
                .into_iter()
                .map(|v| { self.debug_vertice(*v) })
                .sorted()
                .join(", ")
        )
    }

    fn debug_all(&self) -> String {
        self.debug(&self.inner.node_indices().collect_vec())
    }

    fn debug_vertice(&self, vertice: NodeIndex) -> String {
        let w = &self.inner[vertice];
        if let Some(label) = &w.label {
            label.clone()
        } else {
            format!("{}/{}", w.id, w.priority)
        }
    }
}

pub struct Solution<'a> {
    pub even_region: HashSet<&'a MetaData>,
    pub odd_region: HashSet<&'a MetaData>,
    pub strategy: HashMap<usize, Strategy>,
}

impl Solution<'_> {
    fn empty() -> Self {
        Solution {
            even_region: HashSet::new(),
            odd_region: HashSet::new(),
            strategy: HashMap::new(),
        }
    }
}

pub struct Strategy {
    pub winner: Owner,
    pub next_node_id: Option<usize>,
}

impl Display for Solution<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "paritysol {};", self.strategy.len())?;
        for (v, s) in self.strategy.iter().sorted_by_key(|(&k, _)| k) {
            write!(f, "{} {}", v, s.winner)?;

            if let Some(next) = s.next_node_id {
                write!(f, " {}", next)?;
            }

            write!(f, ";\n")?;
        }

        Ok(())
    }
}
