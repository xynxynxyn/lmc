use std::{
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

    // Find all possible markings
    let mut visited = Vec::new();
    let mut queue = Vec::new();
    queue.push(net.initial_marking());

    let mut timestamp = SystemTime::now();
    let start = SystemTime::now();
    while let Some(marking) = queue.pop() {
        visited.push(marking.clone());
        if let Ok(duration) = timestamp.elapsed() {
            if duration >= Duration::from_secs(5) {
                if let Ok(elapsed) = start.elapsed() {
                    println!("{:6}: {:10} markings", elapsed.as_secs(), visited.len());
                }
                timestamp = SystemTime::now();
            }
        }

        let next_markings = marking.next(&net)?;
        for m in next_markings {
            if !visited.contains(&m) && !queue.contains(&m) {
                queue.push(m);
            }
        }
    }

    println!("Took {}s", start.elapsed().unwrap().as_secs());

    let deadlock = visited.iter().filter(|m| m.deadlock(&net).unwrap());
    println!(
        "{} reachable markings, {} deadlock markings",
        visited.len(),
        deadlock.count()
    );

    Ok(())
}
