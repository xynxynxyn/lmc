mod transform;

use crate::transform::petri_to_gnba;
use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use itertools::Itertools;
use ltl::Formula;
use petri::PetriNet;
use std::ffi::OsString;
use std::{
    collections::{HashSet, VecDeque},
    fs,
    time::{Duration, SystemTime},
};
use transform::ltl_to_gnba;

// opt parsing
#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
/// Provides analysis for PetriNets, LTL property verification and various LTL model checking
/// toolings.
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}
#[derive(Subcommand)]
enum Commands {
    /// Analyse the statespace of PetriNets provided by the given files
    Petri {
        /// Number of PNML files which contain PetriNets to be analysed
        file: OsString,
        #[clap(short, long)]
        analyse: bool,
        #[clap(short, long)]
        ltl: Option<OsString>,
    },
    /// Operate on LTL formulas
    LTL {
        /// LTL formulas in prefix notation, for example '& a b' or '| X a G b'
        formula: String,
        #[clap(short, long)]
        /// Convert the LTL formulas to PNF form
        pnf: bool,
        #[clap(short, long)]
        satisfiable: bool,
        /// Check if the provided LTL formula is satisfiable
        #[clap(short, long)]
        /// Generate a NBA from the LTL formula in HOA format
        nba: bool,
        #[clap(short, long)]
        /// Generate a GNBA from the LTL formula in HOA format
        gnba: bool,
        #[clap(short, long)]
        dot: bool,
    },
    Parity {
        /// Parity game file to parse
        file: OsString,
        /// Print the vertices won by each player to stdout
        #[clap(short, long)]
        regions: bool,
        /// Print the strategy derived for the input to stdout
        #[clap(short, long)]
        strategy: bool,
        /// Which algorithm to use
        #[clap(short, long)]
        #[clap(value_enum)]
        algorithm: Option<Algorithm>,
        /// Instead of printing the solution to stdout it is written to the given file instead
        #[clap(short, long)]
        target: Option<OsString>,
    },
}

#[derive(clap::ValueEnum, Clone, Copy)]
enum Algorithm {
    FPI,
    Zielonka,
    Tangle,
    SPM,
}

fn main() -> Result<()> {
    env_logger::init();
    let cli = Cli::parse();

    match &cli.command {
        Commands::Petri { file, analyse, ltl } => {
            if *analyse {
                println!("-- Analysing PNML file '{}'", file.to_string_lossy());
                analyse_petri_net(&file)?;
            }

            if let Some(path) = ltl {
                let file_content = fs::read_to_string(path)?;
                let formulas = ltl::xml::parse(&file_content);
                let net = read_petri(file)?;
                // gnba of the petri net
                let _gnba = petri_to_gnba(net);
                match formulas {
                    Some(formulas) => {
                        for (id, f) in formulas {
                            println!("{}: '{}'", id, f);
                            println!("{}", ltl_to_gnba(&f).hoa());
                        }
                        // Analyse the petri net by creating the intersection
                    }
                    None => println!(
                        "Could not parse formulas from file {}",
                        path.to_string_lossy()
                    ),
                }
            }
        }
        Commands::LTL {
            formula,
            pnf,
            satisfiable,
            nba,
            gnba,
            dot,
        } => {
            let parsed_formula = Formula::parse(formula)?;
            println!("Formula: '{}'", parsed_formula);
            let pnf_formula = parsed_formula.pnf();
            if *pnf {
                println!("PNF: '{}'", pnf_formula);
            }

            if *gnba || *nba || *satisfiable {
                println!("--- Creating GNBA ---");
                let gnba_f = ltl_to_gnba(&pnf_formula);

                if *gnba {
                    println!("--- Generated GNBA ---\n{}", gnba_f.hoa());
                    if *dot {
                        println!("--- GNBA dot ---\n{}", gnba_f.to_dot());
                    }
                }

                if *nba {
                    println!("--- Creating NBA ---");
                    let nba_f = gnba_f.gnba_to_nba();
                    if *nba {
                        println!("--- Generated NBA ---\n{}", nba_f.hoa());
                        if *dot {
                            println!("--- NBA dot ---\n{}", nba_f.to_dot());
                        }
                    }
                }
            }
            if *satisfiable {
                println!("--- Checking Satisfiability ---");
                // Negate the formula and verify it
                let negation = Formula::parse(&format!("!{}", formula))?;
                let trace = ltl_to_gnba(&negation).verify();
                match trace {
                    Ok(_) => println!("False"),
                    Err(trace) => println!("Found counterexample trace:\n{}", trace),
                }
            }
        }
        Commands::Parity {
            file,
            regions,
            strategy,
            algorithm,
            target,
        } => {
            let input = fs::read_to_string(file)?;
            let game = parity::parse_game(&input).context("Could not parse parity game")?;
            let algorithm = algorithm.unwrap_or(Algorithm::FPI);
            let sol = match algorithm {
                Algorithm::FPI => game.fpi(),
                Algorithm::Zielonka => game.zielonka(),
                Algorithm::Tangle => game.tangle(),
                Algorithm::SPM => game.spm(),
            };

            if *regions {
                if !sol.even_region.is_empty() {
                    println!(
                        "won by even: {}",
                        sol.even_region
                            .iter()
                            .sorted_by_key(|m| m.id)
                            .map(|m| match &m.label {
                                Some(label) => format!("{}", label),
                                None => format!("{}/{}", m.id, m.priority),
                            })
                            .collect_vec()
                            .join(" ")
                    );
                }
                if !sol.odd_region.is_empty() {
                    println!(
                        "won by odd: {}",
                        sol.odd_region
                            .iter()
                            .sorted_by_key(|m| m.id)
                            .map(|m| match &m.label {
                                Some(label) => format!("{}", label),
                                None => format!("{}/{}", m.id, m.priority),
                            })
                            .collect_vec()
                            .join(" ")
                    );
                }
            }

            if *strategy {
                if let Some(path) = target {
                    fs::write(path, sol.to_string())?;
                } else {
                    println!("{}", sol)
                }
            }
        }
    }

    Ok(())
}

fn read_petri(path: &OsString) -> petri::Result<PetriNet> {
    let file_content = fs::read_to_string(path)?;
    petri::from_xml(&file_content).into()
}

fn analyse_petri_net(path: &OsString) -> Result<()> {
    let net = read_petri(path)?;

    let start = SystemTime::now();
    // Find all possible markings
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();
    queue.push_back(net.initial_marking());
    visited.insert(net.initial_marking());

    while let Some(marking) = queue.pop_front() {
        let next_markings = net.next_markings(&marking)?;
        for m in next_markings {
            if !visited.contains(&m) {
                visited.insert(m.clone());
                queue.push_back(m);
            }
        }
    }

    let elapsed = start.elapsed().unwrap();
    if elapsed <= Duration::from_millis(1) {
        println!("-- Analysis took {}μs", elapsed.as_micros());
    } else if elapsed <= Duration::from_secs(1) {
        println!("-- Analysis took {}ms", elapsed.as_millis());
    } else {
        println!("-- Analysis took {}s", elapsed.as_secs_f64());
    }

    let deadlock_count = visited.iter().filter(|m| net.deadlock(&m).unwrap()).count();
    println!(
        "Found {} reachable markings, out of which {} are deadlocks",
        visited.len(),
        deadlock_count
    );
    Ok(())
}
