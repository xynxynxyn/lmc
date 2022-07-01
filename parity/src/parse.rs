use crate::{Graph, MetaData, Owner};
use nom::bytes::complete::tag;
use nom::character::complete::{alphanumeric1, char, digit1, multispace1};
use nom::combinator::{map, opt};
use nom::multi::separated_list0;
use nom::sequence::{delimited, tuple};
use nom::IResult;
use std::collections::HashMap;
fn parse_usize(input: &str) -> IResult<&str, usize> {
    map(digit1, |s: &str| {
        s.parse::<usize>().expect("Could not parse usize")
    })(input)
}

// Parsing a game
pub fn parse_game_header(input: &str) -> IResult<&str, usize> {
    map(
        tuple((tag("parity"), multispace1, parse_usize, char(';'))),
        |t| t.2,
    )(input)
}

pub struct GameLine<'a> {
    id: usize,
    priority: usize,
    owner: Owner,
    successors: Vec<usize>,
    label: Option<&'a str>,
}

pub fn parse_game_line(input: &str) -> IResult<&str, GameLine> {
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

    log::info!(
        "parsed parity game with {} vertices: {}",
        number_of_nodes,
        g.debug_all()
    );

    Some(g)
}
