use std::{env, fs};
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

    // Find all possible markings
    let mut visited = Vec::new();
    let mut queue = Vec::new();
    queue.push(net.initial_marking());

    while let Some(marking) = queue.pop() {
        visited.push(marking.clone());
        let next_markings = marking.next(&net)?;
        for m in next_markings {
            if !visited.contains(&m) && !queue.contains(&m) {
                queue.push(m);
            }
        }
    }

    let deadlock: Vec<_> = visited
        .iter()
        .filter(|m| m.deadlock(&net).unwrap())
        .collect();
    println!(
        "{} reachable {}",
        visited.len(),
        if visited.len() < 2 {
            "marking"
        } else {
            "markings"
        }
    );
    for (i, m) in visited.iter().enumerate() {
        println!("\t[{:2}] ({})", i, m.fmt(&net));
    }
    println!(
        "{} deadlock {}",
        deadlock.len(),
        if deadlock.len() < 2 {
            "marking"
        } else {
            "markings"
        }
    );
    for (i, m) in deadlock.iter().enumerate() {
        println!("\t[{:2}] ({})", i, m.fmt(&net));
    }

    Ok(())
}
