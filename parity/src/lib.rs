mod fpi;
mod parse;
use itertools::Itertools;
pub use parse::parse_game;
use std::collections::{HashMap, HashSet};
use std::fmt::Display;
use petgraph::graph::DiGraph;

// The main data structure is a Graph
// Each vertex contains information:
// - What is the priority (a number from 0 to n)
// - What is the
#[derive(Clone, PartialEq, Eq, Hash)]
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

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Owner {
    Odd,
    Even,
}

impl Display for Owner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Owner::Odd => write!(f, "1"),
            Owner::Even => write!(f, "0"),
        }
    }
}

pub struct Graph {
    inner: DiGraph<MetaData, ()>,
}

impl Graph {
    fn new() -> Self {
        Graph {
            inner: DiGraph::new(),
        }
    }
}

pub struct Solution<'a> {
    pub even_region: HashSet<&'a MetaData>,
    pub odd_region: HashSet<&'a MetaData>,
    pub strategy: HashMap<usize, Strategy>,
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
