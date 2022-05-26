mod transform;

use anyhow::Result;
use clap::{Parser, Subcommand};
use ltl::formula::Formula;
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
        files: Vec<OsString>,
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
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Petri { files } => {
            println!("-- Mode: Petrinet Analysis --");
            for path in files {
                println!("-- Analysing PNML file '{}'", path.to_string_lossy());
                analyse_petri_net(&path)?;
            }
        }
        Commands::LTL {
            formula,
            pnf,
            satisfiable,
            nba,
            gnba,
        } => {
            let formula = Formula::parse(formula)?;
            println!("Formula: '{}'", formula);
            let pnf_formula = formula.pnf();
            if *pnf {
                println!("PNF: '{}'", pnf_formula);
            }

            if *gnba || *nba || *satisfiable {
                println!("--- Creating GNBA ---");
                let gnba_f = ltl_to_gnba(&pnf_formula);

                if *gnba {
                    println!("--- Generated GNBA ---\n{}", gnba_f.hoa());
                }

                if *nba || *satisfiable {
                    println!("--- Creating NBA ---");
                    let nba_f = gnba_f.gnba_to_nba();
                    if *nba {
                        println!("--- Generated NBA ---\n{}", nba_f.hoa());
                    }
                    if *satisfiable {
                        println!("--- Checking Satisfiability ---");
                        let trace = nba_f.verify();
                        match trace {
                            Ok(_) => println!("Satisfiable: False"),
                            Err(t) => println!("Satisfiable: True\nTrace: {}", t),
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

fn analyse_petri_net(path: &OsString) -> Result<()> {
    let file_content = fs::read_to_string(path)?;
    let net = petri::from_xml(&file_content)?;

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
