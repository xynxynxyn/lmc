mod parser;

pub use parser::from_xml;
use std::{collections::HashMap, fmt};

use crate::error::{Error, Result};

#[derive(Debug)]
struct Place {
    id: String,
    initial_marking: usize,
}

#[derive(Debug)]
struct Transition {
    inputs: Vec<usize>,
    outputs: Vec<usize>,
}

#[derive(Debug)]
pub struct PetriNet {
    places: Vec<Place>,
    transitions: Vec<Transition>,
    place_labels: HashMap<String, usize>,
    transition_labels: HashMap<String, usize>,
}

impl PetriNet {
    fn new() -> Self {
        PetriNet {
            places: vec![],
            transitions: vec![],
            place_labels: HashMap::new(),
            transition_labels: HashMap::new(),
        }
    }

    fn add_place(&mut self, place: String, initial_marking: usize) -> Result<()> {
        if self.place_labels.contains_key(&place) {
            Err(Error::DuplicatePlace(place))
        } else {
            let index = self.places.len();
            self.places.push(Place {
                id: place.clone(),
                initial_marking,
            });
            self.place_labels.insert(place, index);
            Ok(())
        }
    }

    fn add_transition(&mut self, transition: String) -> Result<()> {
        if self.transition_labels.contains_key(&transition) {
            Err(Error::DuplicateTransition(transition))
        } else {
            let index = self.transitions.len();
            self.transitions.push(Transition {
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
            self.transition_labels.get(&target),
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
            self.transition_labels.get(&source),
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
            markings: self.places.iter().map(|p| p.initial_marking).collect(),
        }
    }

    pub fn next_markings(&self, marking: &Marking) -> Result<Vec<Marking>> {
        marking.next(self)
    }

    pub fn deadlock(&self, marking: &Marking) -> Result<bool> {
        marking.deadlock(self)
    }
}

/// Maps stores the number of tokens for each place in a net
#[derive(Clone, Debug)]
pub struct Marking {
    markings: Vec<usize>,
}

impl PartialEq for Marking {
    fn eq(&self, other: &Self) -> bool {
        if self.markings.len() != other.markings.len() {
            return false;
        }

        self.markings
            .iter()
            .zip(other.markings.iter())
            .filter(|&(a, b)| a == b)
            .count()
            == self.markings.len()
    }
}

impl fmt::Display for Marking {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.markings)
    }
}

impl Marking {
    /// Calculate the next marking
    /// Will panic if indices do not match ( but this shouldn't happen as long as the underlying
    /// petri net never gets mutated )
    fn next(&self, net: &PetriNet) -> Result<Vec<Marking>> {
        if self.markings.len() != net.places.len() {
            return Err(Error::InvalidIndex);
        }
        // Get transitions which are active
        let active_transitions = net.transitions.iter().filter(|t| {
            t.inputs
                .iter()
                .fold(true, |acc, i| if acc { self.markings[*i] > 0 } else { acc })
        });

        Ok(active_transitions
            .map(|t| {
                let mut marking = self.clone();
                for &i in &t.inputs {
                    marking.markings[i] -= 1;
                }
                for &i in &t.outputs {
                    marking.markings[i] += 1;
                }
                marking
            })
            .collect())
    }

    fn deadlock(&self, net: &PetriNet) -> Result<bool> {
        self.next(net).map(|m| m.is_empty())
    }
}
