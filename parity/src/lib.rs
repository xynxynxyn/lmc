use itertools::Itertools;
use nom::bytes::complete::tag;
use nom::character::complete::{alphanumeric1, char, digit1, multispace1};
use nom::combinator::{map, opt};
use nom::multi::separated_list0;
use nom::sequence::{delimited, tuple};
use nom::IResult;
use std::collections::{BTreeSet, HashMap, HashSet};
use std::fmt::Display;

use petgraph::graph::{DiGraph, NodeIndex};
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

pub fn parse_game(game: &str) -> Option<Graph> {
    let mut g = Graph::new();

    let lines: Vec<_> = game.lines().collect();

    if lines.is_empty() {
        return None;
    }

    let number_of_nodes = parse_game_header(lines[0]).ok()?.1;

    let mut nodes = HashMap::new();
    for i in 0..number_of_nodes {
        let node_index = g.inner.add_node(MetaData::new(i));
        nodes.insert(i, node_index);
    }

    for line in lines[1..].iter() {
        let data: GameLine = parse_game_line(line).ok()?.1;
        let node_index = nodes[&data.id];
        let mut meta_data = g
            .inner
            .node_weight_mut(node_index)
            .expect("Could not find node with given index");
        meta_data.label = data.label.map(String::from);
        meta_data.owner = data.owner;
        meta_data.priority = data.priority;

        for successor in data.successors {
            let successor_index = nodes[&successor];
            g.inner.add_edge(node_index, successor_index, ());
        }
    }

    Some(g)
}

impl Graph {
    fn new() -> Self {
        Graph {
            inner: DiGraph::new(),
        }
    }

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

fn parse_usize(input: &str) -> IResult<&str, usize> {
    map(digit1, |s: &str| {
        s.parse::<usize>().expect("Could not parse usize")
    })(input)
}

// Parsing a game
fn parse_game_header(input: &str) -> IResult<&str, usize> {
    map(
        tuple((tag("parity"), multispace1, parse_usize, char(';'))),
        |t| t.2,
    )(input)
}

struct GameLine<'a> {
    id: usize,
    priority: usize,
    owner: Owner,
    successors: Vec<usize>,
    label: Option<&'a str>,
}

fn parse_game_line(input: &str) -> IResult<&str, GameLine> {
    map(
        tuple((
            parse_usize,
            multispace1,
            parse_usize,
            multispace1,
            parse_usize,
            multispace1,
            separated_list0(tag(","), parse_usize),
            opt(tuple((
                multispace1,
                delimited(tag("\""), alphanumeric1, tag("\"")),
            ))),
        )),
        |t| GameLine {
            id: t.0,
            priority: t.2,
            owner: match t.4 {
                0 => Owner::Even,
                1 => Owner::Odd,
                _ => panic!("Expected 0 or 1, cannot parse owner"),
            },
            successors: t.6,
            label: t.7.map(|l| l.1),
        },
    )(input)
}
