use nom::bytes::complete::tag;
use nom::character::complete::{alphanumeric1, char, digit1, multispace1};
use nom::combinator::{map, opt};
use nom::multi::separated_list0;
use nom::sequence::tuple;
use nom::IResult;
use std::collections::{BTreeSet, HashMap};

use petgraph::graph::{DiGraph, NodeIndex};
// The main data structure is a Graph
// Each vertex contains information:
// - What is the priority (a number from 0 to n)
// - What is the
struct MetaData {
    id: usize,
    label: Option<String>,
    owner: Owner,
    priority: usize,
}

impl MetaData {
    fn new(id: usize) -> Self {
        MetaData {
            id,
            label: None,
            owner: Owner::One,
            priority: 0,
        }
    }
}

enum Owner {
    One,
    Two,
}

pub struct Graph {
    inner: DiGraph<MetaData, ()>,
}

impl Graph {
    pub fn parse_game(game: &str) -> Option<Self> {
        let mut g = Self::new();

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
    fn onestep(&self, v: NodeIndex, z: &BTreeSet<NodeIndex>) -> usize {
        let p = self
            .inner
            .node_weight(v)
            .expect("Could not find node with given weight")
            .priority;

        if p % 2 == 0 {
            if self
                .inner
                .neighbors(v)
                .into_iter()
                .any(|n| self.winner(n, z) == 0)
            {
                0
            } else {
                1
            }
        } else {
            if self
                .inner
                .neighbors(v)
                .into_iter()
                .any(|n| self.winner(n, z) == 1)
            {
                1
            } else {
                0
            }
        }
    }

    pub fn fpi(&self) -> (BTreeSet<NodeIndex>, BTreeSet<NodeIndex>) {
        let mut z = BTreeSet::new();
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
                .filter(|v| z.contains(v))
                .filter(|v| self.onestep(*v, &z) != parity)
                .collect();
            if y.is_empty() {
                z = z
                    .intersection(&y)
                    .cloned()
                    .filter(|v| {
                        self.inner
                            .node_weight(*v)
                            .expect("Could not find node weight for given index")
                            .priority
                            >= p
                    })
                    .collect();
                p = 0;
            } else {
                p += 1;
            }
        }
        self.inner
            .node_indices()
            .into_iter()
            .partition(|v| self.winner(*v, &z) == 0)
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
            opt(tuple((multispace1, alphanumeric1))),
        )),
        |t| GameLine {
            id: t.0,
            priority: t.2,
            owner: match t.4 {
                0 => Owner::One,
                1 => Owner::Two,
                _ => panic!("Expected 0 or 1, cannot parse owner"),
            },
            successors: t.6,
            label: t.7.map(|l| l.1),
        },
    )(input)
}
