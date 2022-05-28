use std::fmt::Display;

use quick_xml::de::from_str;
use serde_derive::Deserialize;

use crate::{error::Error, Formula};

pub fn parse(input: &str) -> Option<Vec<(String, Formula)>> {
    let properties = from_str::<PropertySet>(input).ok()?.properties;

    Some(
        properties
            .into_iter()
            .map(|p| {
                (
                    p.id,
                    property_to_formula(p.formula)
                        .expect(&format!("Could not parse input {}", input)),
                )
            })
            .collect(),
    )
}

fn property_to_formula(base: AllPathFormula) -> Result<Formula, Error> {
    let raw = base.all_paths.root_formula.to_string();
    Formula::parse(&raw)
}

#[derive(Debug, Deserialize)]
struct PropertySet {
    #[serde(rename = "property")]
    properties: Vec<Property>,
}

#[derive(Debug, Deserialize)]
struct Property {
    id: String,
    formula: AllPathFormula,
}

#[derive(Debug, Deserialize)]
struct AllPathFormula {
    #[serde(rename = "all-paths")]
    all_paths: AllPaths,
}

#[derive(Debug, Deserialize)]
struct AllPaths {
    #[serde(rename = "$value")]
    root_formula: BooleanFormula,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum BooleanFormula {
    Finally {
        #[serde(rename = "$value")]
        inner: Box<BooleanFormula>,
    },
    Globally {
        #[serde(rename = "$value")]
        inner: Box<BooleanFormula>,
    },
    Next {
        #[serde(rename = "$value")]
        inner: Box<BooleanFormula>,
    },
    Negation {
        #[serde(rename = "$value")]
        inner: Box<BooleanFormula>,
    },
    Conjunction {
        #[serde(rename = "$value")]
        inner: Vec<BooleanFormula>,
    },
    Disjunction {
        #[serde(rename = "$value")]
        inner: Vec<BooleanFormula>,
    },
    Until {
        before: Before,
        reach: Reach,
    },
    #[serde(rename = "is-fireable")]
    Atom(Transitions),
}

impl Display for BooleanFormula {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Finally { inner } => write!(f, "F {}", inner),
            Self::Globally { inner } => write!(f, "G {}", inner),
            Self::Next { inner } => write!(f, "X {}", inner),
            Self::Negation { inner } => write!(f, "!{}", inner),
            c @ Self::Conjunction { inner } => {
                if inner.len() <= 1 {
                    panic!(
                        "Conjunction: {:?} does not have at least two subformulas",
                        c
                    )
                }
                for _ in 0..inner.len() - 1 {
                    write!(f, "{}", "& ")?;
                }
                write!(
                    f,
                    "{}",
                    inner
                        .iter()
                        .map(Self::to_string)
                        .collect::<Vec<_>>()
                        .join(" ")
                )
            }
            d @ Self::Disjunction { inner } => {
                if inner.len() <= 1 {
                    panic!(
                        "Disjunction: {:?} does not have at least two subformulas",
                        d
                    )
                }
                for _ in 0..inner.len() - 1 {
                    write!(f, "{}", "| ")?;
                }
                write!(
                    f,
                    "{}",
                    inner
                        .iter()
                        .map(Self::to_string)
                        .collect::<Vec<_>>()
                        .join(" ")
                )
            }
            Self::Until { before, reach } => write!(f, "U {} {}", before.inner, reach.inner),
            Self::Atom(transitions) => {
                for _ in 0..transitions.transitions.len() - 1 {
                    write!(f, "& ")?;
                }

                write!(f, "{}", transitions.transitions.join(" "))
            }
        }
    }
}

#[derive(Debug, Deserialize)]
struct Before {
    #[serde(rename = "$value")]
    inner: Box<BooleanFormula>,
}

#[derive(Debug, Deserialize)]
struct Reach {
    #[serde(rename = "$value")]
    inner: Box<BooleanFormula>,
}

#[derive(Debug, Deserialize)]
struct Transitions {
    #[serde(rename = "transition")]
    transitions: Vec<String>,
}
