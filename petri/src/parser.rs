use super::PetriNet;
use crate::error::Result;
use serde_derive::Deserialize;
use serde_xml_rs::from_str;

#[derive(Debug, Deserialize)]
struct Pnml {
    net: Net,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Net {
    #[serde(rename = "page")]
    pages: Vec<Page>,
}

#[derive(Debug, Deserialize)]
struct Page {
    #[serde(rename = "place")]
    places: Vec<Place>,
    #[serde(rename = "transition")]
    transitions: Vec<Transition>,
    #[serde(rename = "arc")]
    arcs: Vec<Arc>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Place {
    id: String,
    initial_marking: Option<InitialMarking>,
}

#[derive(Debug, Deserialize)]
struct InitialMarking {
    text: usize,
}

#[derive(Debug, Deserialize)]
struct Transition {
    id: String,
}

#[derive(Debug, Deserialize)]
struct Arc {
    source: String,
    target: String,
}

pub fn from_xml(input: &str) -> Result<PetriNet> {
    let raw_pnml: Pnml = from_str(input)?;
    let raw_net = raw_pnml.net;
    let mut net = PetriNet::new();

    // collect all the pages into a single tuple of elements
    let (places, transitions, arcs) = raw_net
        .pages
        .into_iter()
        .reduce(|mut accum, mut page| {
            accum.arcs.append(&mut page.arcs);
            accum.places.append(&mut page.places);
            accum.transitions.append(&mut page.transitions);
            accum
        })
        .map(|p| (p.places, p.transitions, p.arcs))
        .unwrap_or((vec![], vec![], vec![]));

    for place in places {
        net.add_place(
            place.id,
            place
                .initial_marking
                .unwrap_or(InitialMarking { text: 0 })
                .text,
        )?;
    }

    for transition in transitions {
        net.add_transition(transition.id)?;
    }

    for arc in arcs {
        net.add_arc(arc.source, arc.target)?;
    }

    Ok(net)
}
