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

#[derive(Clone, Debug)]
pub struct Buchi {
    /// A State and it's transitions
    /// These transitions take a word as input and return a set of new states
    states: HashMap<State, HashMap<Word, HashSet<State>>>,
    accepting_states: HashSet<State>,
    initial_states: HashSet<State>,
}

#[derive(Debug, Eq, Clone, Hash, PartialEq)]
pub struct Word {
    pub id: String,
}

#[derive(Debug, Eq, Clone, Hash, PartialEq)]
pub struct State {
    pub id: String,
}

#[derive(Debug)]
pub struct Trace {
    pub words: Vec<Word>,
    pub omega_words: Vec<Word>,
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
    pub fn set_initial_state(&mut self, state: &State) {
        let state = state.clone();
        self.initial_states.insert(state.clone());
        if !self.states.contains_key(&state) {
            self.add_state(&state);
        }
    }

    /// Adds the state if it doesn't already exist
    pub fn set_accepting_state(&mut self, state: &State) {
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

    pub fn accepting_states(&self) -> HashSet<State> {
        self.accepting_states.clone()
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
            visited.insert(initial_state, vec![]);
            queue.push(initial_state);

            while let Some(state) = queue.pop() {
                if accepting.contains(state) {
                    // Found a counter example, return the trace and calculate an omega trace
                    let scc = sccs
                        .iter()
                        .filter(|c| c.contains(state))
                        .collect::<Vec<_>>()[0];

                    let trace = visited.remove(state).unwrap();
                    let omega_trace = self.constrained_cycle_searcher(state, scc).unwrap();

                    return Err(Trace::new(trace, omega_trace));
                }

                for transition in self.states.get(state) {
                    for (word, successors) in transition {
                        for successor in successors {
                            if !visited.contains_key(successor) {
                                // Create a new trace for the newly discovered state by copying the previous one
                                let mut new_trace = visited.get(state).unwrap().clone();
                                new_trace.push(word.clone());
                                visited.insert(successor, new_trace);
                                queue.push(successor);
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn constrained_cycle_searcher(
        &self,
        initial_state: &State,
        states: &HashSet<State>,
    ) -> Option<Vec<Word>> {
        let mut queue = vec![];
        let mut visited = HashMap::new();
        visited.insert(initial_state, vec![]);
        queue.push(initial_state);

        while let Some(state) = queue.pop() {
            for transition in self.states.get(state) {
                for (word, successors) in transition {
                    for successor in successors.iter().filter(|s| states.contains(s)) {
                        if successor == initial_state {
                            // Found the initial state again, return the trace
                            let mut trace = visited.remove(state).unwrap();
                            trace.push(word.clone());
                            return Some(trace);
                        }

                        let mut new_trace = visited.get(state).unwrap().clone();
                        new_trace.push(word.clone());
                        visited.insert(successor, new_trace);
                        queue.push(successor);
                    }
                }
            }
        }

        None
    }

    pub fn gnba_to_nba(&self) -> Self {
        // If the accepting states are empty or there's only one it doesn't matter what you do, just return the whole gnba since it's already an nba
        if self.accepting_states.len() <= 1 {
            return self.clone();
        }
        // Clone the accepting states into a vec for deterministic ordering
        let accepting_states: Vec<_> = self.accepting_states.clone().into_iter().collect();

        let mut nba = Buchi::new();
        // Duplicate the statespace
        for (i, accepting) in accepting_states.iter().enumerate() {
            // Create a copy of the statespace for every accepting state and rename them to s0_0, s0_1, s0_2 etc for each iteration
            let mut new_states: HashMap<State, HashMap<Word, HashSet<State>>> = self
                .states
                .clone()
                .into_iter()
                .map(|(mut k, mut v)| {
                    // Rename the source state
                    k.id = format!("{}_{}", k.id, i);
                    // Rename the target states while leaving the word the same
                    for targets in v.values_mut() {
                        *targets = targets
                            .iter()
                            .map(|s| State::new(format!("{}_{}", s.id, i)))
                            .collect();
                    }
                    (k, v)
                })
                .collect();
            // Map the transitions of the current accepting state to point towards the next one (potentially the first)
            let next_index = (i + 1) % accepting_states.len();
            new_states
                .entry(State::new(format!("{}_{}", accepting.id, i)))
                .and_modify(|transitions| {
                    for targets in transitions.values_mut() {
                        *targets = targets
                            .iter()
                            .map(|_| State::new(format!("{}_{}", accepting.id, next_index)))
                            .collect();
                    }
                });

            nba.states.extend(new_states);
            // Set the last accepting state
            if i == accepting_states.len() - 1 {
                nba.set_accepting_state(&State::new(format!("{}_{}", accepting.id, i)));
            }
        }
        // Copy the initial states
        for initial_state in &self.initial_states {
            nba.set_initial_state(&State::new(format!("{}_{}", initial_state.id, 0)))
        }

        nba
    }
}

impl Display for Buchi {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "Initial States: ({})",
            self.initial_states
                .iter()
                .map(|s| s.id.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        )?;
        writeln!(
            f,
            "Accepting States: ({})",
            self.accepting_states
                .iter()
                .map(|s| s.id.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        )?;
        writeln!(f, "Transitions:")?;
        for (s, transitions) in &self.states {
            for (word, targets) in transitions {
                for t in targets {
                    writeln!(f, "{} --({})--> {}", s.id, word.id, t.id)?;
                }
            }
        }
        Ok(())
    }
}

impl Word {
    pub fn new<T: ToString>(id: T) -> Self {
        Word { id: id.to_string() }
    }
}

impl State {
    pub fn new<T: ToString>(id: T) -> Self {
        State { id: id.to_string() }
    }
}

impl Trace {
    pub fn new(words: Vec<Word>, omega_words: Vec<Word>) -> Self {
        Trace { words, omega_words }
    }
}

impl Display for Trace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "({})({})ω",
            self.words
                .iter()
                .map(|w| w.id.as_str())
                .collect::<Vec<&str>>()
                .join(","),
            self.omega_words
                .iter()
                .map(|w| w.id.as_str())
                .collect::<Vec<&str>>()
                .join(",")
        )?;
        Ok(())
    }
}
