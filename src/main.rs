use std::{
    collections::{HashSet, VecDeque},
    env, fs,
    time::{Duration, SystemTime},
};
mod error;
mod petri;

fn main() -> error::Result<()> {
    let args: Vec<_> = env::args().collect();
    let file_path = args
        .get(1)
        .map(|s| s.as_str())
        .unwrap_or("inputs/philosophers/Philosophers-5.pnml");
    let file_content = fs::read_to_string(file_path)?;
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
        println!("Took {}μs", elapsed.as_micros());
    } else if elapsed <= Duration::from_secs(1) {
        println!("Took {}ms", elapsed.as_millis());
    } else {
        println!("Took {}s", elapsed.as_secs_f64());
    }

    let deadlock_count = visited.iter().filter(|m| net.deadlock(&m).unwrap()).count();
    println!(
        "{} reachable markings, {} deadlock markings",
        visited.len(),
        deadlock_count
    );
    if deadlock_count > 0 {
        println!("deadlock markings:");
        for m in visited.iter().filter(|m| net.deadlock(&m).unwrap()) {
            println!("  ({})", net.fmt_marking(&m).unwrap());
        }
    }

    Ok(())
}
