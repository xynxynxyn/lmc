mod error;
mod parser;

use bimap::BiMap;
use bitvec::prelude::BitVec;
pub use error::{Error, Result};
pub use parser::from_xml;
use std::collections::HashMap;

struct Place {
    initial_marking: usize,
}

#[derive(Debug)]
struct Transition {
    label: String,
    inputs: Vec<usize>,
    outputs: Vec<usize>,
}

pub struct PetriNet {
    places: Vec<Place>,
    transitions: Vec<Transition>,
    place_labels: HashMap<String, usize>,
    transition_labels: BiMap<String, usize>,
}

impl PetriNet {
    fn new() -> Self {
        PetriNet {
            places: vec![],
            transitions: vec![],
            place_labels: HashMap::new(),
            transition_labels: BiMap::new(),
        }
    }

    fn add_place(&mut self, place: String, initial_marking: usize) -> Result<()> {
        if self.place_labels.contains_key(&place) {
            Err(Error::DuplicatePlace(place))
        } else {
            let index = self.places.len();
            self.places.push(Place { initial_marking });
            self.place_labels.insert(place, index);
            Ok(())
        }
    }

    fn add_transition(&mut self, transition: String) -> Result<()> {
        if self.transition_labels.contains_left(&transition) {
            Err(Error::DuplicateTransition(transition))
        } else {
            let index = self.transitions.len();
            self.transitions.push(Transition {
                label: transition.clone(),
                inputs: vec![],
                outputs: vec![],
            });
            self.transition_labels.insert(transition, index);
            Ok(())
        }
    }

    fn add_arc(&mut self, source: String, target: String) -> Result<()> {
        if let (Some(place_index), Some(transition_index)) = (
            self.place_labels.get(&source),
            self.transition_labels.get_by_left(&target),
        ) {
            // Source is a place
            // Target is a transition
            self.transitions
                .get_mut(*transition_index)
                .ok_or(Error::InvalidIndex)?
                .inputs
                .push(*place_index);
            Ok(())
        } else if let (Some(transition_index), Some(place_index)) = (
            self.transition_labels.get_by_left(&source),
            self.place_labels.get(&target),
        ) {
            // Source is a transition
            // Target is a place
            self.transitions
                .get_mut(*transition_index)
                .ok_or(Error::InvalidIndex)?
                .outputs
                .push(*place_index);
            Ok(())
        } else {
            Err(Error::InvalidArc(source, target))
        }
    }

    pub fn initial_marking(&self) -> Marking {
        Marking {
            markings: self.places.iter().map(|p| p.initial_marking > 0).collect(),
        }
    }

    pub fn transitions<'a>(&'a self, marking: &'a Marking) -> Result<Vec<(&'a str, Marking)>> {
        marking.next(self)
    }

    pub fn next_markings(&self, marking: &Marking) -> Result<Vec<Marking>> {
        marking
            .next(self)
            .map(|inner| inner.into_iter().map(|e| e.1).collect())
    }

    pub fn deadlock(&self, marking: &Marking) -> Result<bool> {
        marking.deadlock(self)
    }
}

/// Maps stores the number of tokens for each place in a net
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Marking {
    markings: BitVec,
}

impl Marking {
    /// Calculate the next marking
    /// Will panic if indices do not match ( but this shouldn't happen as long as the underlying
    /// petri net never gets mutated )
    fn next<'a>(&'a self, net: &'a PetriNet) -> Result<Vec<(&'a str, Marking)>> {
        if self.markings.len() != net.places.len() {
            return Err(Error::InvalidIndex);
        }
        // Get transitions which are active
        let active_transitions = net.transitions.iter().filter(|t| {
            t.inputs
                .iter()
                .fold(true, |acc, i| if acc { self.markings[*i] } else { acc })
        });

        Ok(active_transitions
            .map(|t| {
                let mut marking = self.clone();
                for &i in &t.inputs {
                    marking.markings.set(i, false);
                }
                for &i in &t.outputs {
                    marking.markings.set(i, true);
                }
                (t.label.as_str(), marking)
            })
            .collect())
    }

    pub fn active_transitions<'a>(&'a self, net: &'a PetriNet) -> Vec<&'a str> {
        net.transitions
            .iter()
            .filter(|t| {
                t.inputs
                    .iter()
                    .fold(true, |acc, i| if acc { self.markings[*i] } else { acc })
            })
            .map(|t| t.label.as_str())
            .collect()
    }

    fn deadlock(&self, net: &PetriNet) -> Result<bool> {
        self.next(net).map(|m| m.is_empty())
    }
}
