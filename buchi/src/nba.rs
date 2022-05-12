use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
};
// A buchi automaton consists of 5 elements:
// - Q: set of states
// - E: an alphabet
// - d: a transition function QxE -> 2^Q
// - Q_0: set of initial states
// - F: set of acceptance states

pub struct Buchi {
    /// A State and it's transitions
    /// These transitions take a word as input and return a set of new states
    states: HashMap<State, HashMap<Word, HashSet<State>>>,
    accepting_states: HashSet<State>,
    initial_states: HashSet<State>,
}

#[derive(Debug, Eq, Clone, Hash, PartialEq)]
pub struct Word {
    id: String,
}

#[derive(Debug, Eq, Clone, Hash, PartialEq)]
pub struct State {
    id: String,
}

#[derive(Debug)]
pub struct Trace {
    words: Vec<Word>,
    w_words: Vec<Word>,
}

impl Buchi {
    pub fn new() -> Self {
        Buchi {
            states: HashMap::new(),
            accepting_states: HashSet::new(),
            initial_states: HashSet::new(),
        }
    }

    fn add_state(&mut self, state: &State) {
        let state = state.clone();
        self.states.insert(state.clone(), HashMap::new());
    }

    /// Adds the state if it doesn't already exist
    pub fn initial_state(&mut self, state: &State) {
        let state = state.clone();
        self.initial_states.insert(state.clone());
        if !self.states.contains_key(&state) {
            self.add_state(&state);
        }
    }

    /// Adds the state if it doesn't already exist
    pub fn accepting_states(&mut self, state: &State) {
        let state = state.clone();
        self.accepting_states.insert(state.clone());
        if !self.states.contains_key(&state) {
            self.add_state(&state);
        }
    }

    /// Add a transition from Source to Target with label Word.
    /// If the Source is not present it will be created.
    /// If the Target is not present it will be created.
    pub fn add_transition(&mut self, source: &State, target: &State, word: &Word) {
        // idc about the borrow checker
        let source = source.clone();
        let target = target.clone();
        let word = word.clone();

        // Add the target to the states if it doesn't already exist
        if !self.states.contains_key(&target) {
            self.states.insert(target.clone(), HashMap::new());
        }
        // Insert the necessary transition information
        self.states
            .entry(source)
            .or_insert(HashMap::new())
            .entry(word)
            .or_insert(HashSet::new())
            .insert(target);
    }

    pub fn transitions(&self, state: &State) -> Option<&HashMap<Word, HashSet<State>>> {
        self.states.get(state)
    }

    pub fn states(&self) -> HashSet<State> {
        self.states.keys().map(|s| s.clone()).collect()
    }

    /// Return a set of strongly connected components using Tarjan's algorithm
    pub fn tarjans(&self) -> Vec<HashSet<State>> {
        let mut index = 0;
        let mut stack = Vec::new();
        let mut colors = HashMap::new();
        let mut components = Vec::new();

        for state in &self.states() {
            if !colors.contains_key(state) {
                let mut found_components = self.tarjans_strongconnect(
                    state,
                    self.get_successors(state),
                    &mut stack,
                    &mut colors,
                    &mut index,
                );
                components.append(&mut found_components);
            }
        }

        println!("Final set of components: {:?}", components);
        components
    }

    fn tarjans_strongconnect<'a>(
        &'a self,
        state: &'a State,
        successors: HashSet<&'a State>,
        stack: &mut Vec<&'a State>,
        colors: &mut HashMap<State, (i32, i32)>,
        index: &mut i32,
    ) -> Vec<HashSet<State>> {
        let mut components = vec![];
        colors.insert(state.clone(), (*index, *index));
        *index += 1;
        stack.push(state);

        for successor in successors {
            if !colors.contains_key(successor) {
                // Collect the components found
                let mut found_components = self.tarjans_strongconnect(
                    successor,
                    self.get_successors(successor),
                    stack,
                    colors,
                    index,
                );
                components.append(&mut found_components);

                let state_cols = *colors.get(state).unwrap();
                let successor_cols = *colors.get(successor).unwrap();
                colors.insert(
                    state.clone(),
                    (state_cols.0, std::cmp::min(state_cols.1, successor_cols.1)),
                );
            } else if stack.contains(&successor) {
                let state_cols = *colors.get(state).unwrap();
                let successor_cols = *colors.get(successor).unwrap();
                colors.insert(
                    state.clone(),
                    (state_cols.0, std::cmp::min(state_cols.1, successor_cols.0)),
                );
            }
        }

        let state_cols = *colors.get(state).unwrap();
        if state_cols.0 == state_cols.1 {
            let mut component = HashSet::new();
            while let Some(w) = stack.pop() {
                component.insert(w.clone());
                if w == state {
                    break;
                }
            }
            println!("Constructed component {:?}", component);
            components.push(component);
        }
        components
    }

    fn get_successors(&self, state: &State) -> HashSet<&State> {
        match self.states.get(state) {
            Some(s) => s.values().flatten().collect(),
            None => HashSet::new(),
        }
    }

    /// Verify that there exists no trace which satisfies the automaton
    /// If there exists a counter example give one back
    pub fn verify(&self) -> Result<(), Trace> {
        // Gather all the final states which are contained in a non trivial SCC
        let sccs: Vec<_> = self.tarjans().into_iter().filter(|c| c.len() > 1).collect();
        let accepting: HashSet<_> = self
            .accepting_states
            .iter()
            .filter(|&s| {
                for c in &sccs {
                    if c.contains(s) {
                        return true;
                    }
                }
                return false;
            })
            .collect();

        // If we can reach any of these accepting states we have found a counter example
        let mut visited = HashMap::new();

        for initial_state in &self.initial_states {
            // Do DFS for every initial_state in the list
            // Except when we already visited it
            if visited.contains_key(initial_state) {
                continue;
            }

            let mut queue = vec![];
            visited.insert(initial_state.clone(), Trace::new(vec![]));
            queue.push(initial_state);

            while let Some(state) = queue.pop() {
                if accepting.contains(state) {
                    // Found a counter example, return the trace for now (no omega yet)
                    return Err(visited.remove(state).unwrap());
                }

                for transition in self.states.get(state) {
                    for (word, successors) in transition {
                        for successor in successors {
                            if !visited.contains_key(successor) {
                                // Create a new trace for the newly discovered state by copying the previous one
                                let mut new_trace = visited.get(state).unwrap().words.clone();
                                new_trace.push(word.clone());
                                visited.insert(successor.clone(), Trace::new(new_trace));
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

impl Word {
    pub fn new(id: String) -> Self {
        Word { id }
    }
}

impl State {
    pub fn new(id: String) -> Self {
        State { id }
    }
}

impl Trace {
    pub fn new(words: Vec<Word>) -> Self {
        Trace {
            words,
            w_words: vec![],
        }
    }
}

impl Display for Trace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "({})",
            self.words
                .iter()
                .map(|w| w.id.as_str())
                .collect::<Vec<&str>>()
                .join(",")
        )?;
        Ok(())
    }
}
