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
    Analyse {
        /// Number of PNML files which contain PetriNets to be analysed
        files: Vec<OsString>,
    },
    /// Convert a list of LTL formula to PNF form
    PNF {
        /// LTL formulas in prefix notation, for example '& a b' or '| X a G b'
        formulas: Vec<String>,
    },
    /// Transform an LTL formula to a corresponding GNBA
    GNBA {
        /// The formula to transform
        formula: String,
    },
    /// Transform an LTL formula to a corresponding NBA
    NBA {
        /// The formula to transform
        formula: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Analyse { files } => {
            println!("-- Mode: Petrinet Analysis --");
            for path in files {
                println!("-- Analysing PNML file '{}'", path.to_string_lossy());
                analyse_petri_net(&path)?;
            }
        }
        Commands::PNF { formulas } => {
            let pnf = formulas.into_iter().map(|s| Formula::parse(s));
            for p in pnf.map(|f| f.map(|f| f.pnf())) {
                match p {
                    Ok(formula) => println!("{}", formula),
                    Err(e) => println!("Err: {}", e),
                }
            }
        }
        Commands::GNBA { formula } => {
            let formula = Formula::parse(formula)?;
            println!("{}", ltl_to_gnba(&formula).hoa());
        }
        Commands::NBA { formula } => {
            let formula = Formula::parse(formula)?;
            println!("{}", ltl_to_gnba(&formula).gnba_to_nba().hoa());
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
