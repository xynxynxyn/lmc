mod fpi;
mod parse;
mod zielonka;
use itertools::Itertools;
pub use parse::parse_game;
use petgraph::graph::NodeIndex;
use petgraph::stable_graph::StableDiGraph;
use std::collections::{HashMap, HashSet};
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
