use bimap::BiMap;
use itertools::Itertools;
use std::fmt::Write;
use std::{
    collections::{BTreeSet, HashMap, HashSet},
    fmt::Display,
};
// A buchi automaton consists of 5 elements:
// - Q: set of states
// - E: an alphabet
// - d: a transition function QxE -> 2^Q
// - Q_0: set of initial states
// - F: set of acceptance states

/// A non-deterministic buchi automata (nba)
/// States are constructed with the automata and must only be used with the automata it is generated from.
/// If States are constructed in another way and used with an automata this can cause panics or incorrect behavior.
#[derive(Clone, Debug)]
pub struct Buchi {
    // A State and it's transitions
    // These transitions take a word as input and return a set of new states
    states: HashMap<State, HashMap<Word, HashSet<State>>>,
    accepting_sets: HashSet<BTreeSet<State>>,
    initial_states: HashSet<State>,
    labels: HashMap<State, String>,
    size: usize,
}

#[derive(Debug, Eq, Clone, Hash, PartialEq)]
pub struct Word {
    pub id: String,
}

#[derive(Debug, Eq, Clone, Copy, Hash, PartialEq, PartialOrd, Ord)]
pub struct State {
    pub id: usize,
}

#[derive(Debug)]
pub struct Trace {
    pub words: Vec<Word>,
    pub omega_words: Vec<Word>,
}

pub struct Transition<'a> {
    pub from: &'a str,
    pub from_state: State,
    pub to: &'a str,
    pub to_state: State,
    pub label: &'a str,
}

// Formatting
impl Buchi {
    /// Tranform the automataon into HOA formatted string
    pub fn hoa(&self) -> String {
        let version = "HOA: v1".into();
        let states = format!("States: {}", self.states.len());
        let start = if self.initial_states.is_empty() {
            "".into()
        } else {
            format!(
                "Start: {}",
                self.initial_states
                    .iter()
                    .sorted_by_key(|s| s.id)
                    .map(|s| s.id.to_string())
                    .collect::<Vec<_>>()
                    .join(" & ")
            )
        };
        let acceptance_sets: BiMap<_, _> = self.accepting_sets.iter().enumerate().collect();

        // If there are 0 accepting states any run is accepted since this is a GNBA
        let acceptance = if acceptance_sets.len() > 0 {
            format!(
                "Acceptance: {} {}",
                acceptance_sets.len(),
                acceptance_sets
                    .iter()
                    .sorted_by_key(|(id, _)| *id)
                    .map(|(mapped_id, _)| format!("Inf({})", mapped_id))
                    .collect::<Vec<_>>()
                    .join("&")
            )
        } else {
            "Acceptance: 0 t".into()
        };

        let header = vec![version, states, start, acceptance].join("\n");

        let mut states = Vec::with_capacity(self.states.len());

        for (state, transitions) in self.states.iter().sorted_by_key(|(s, _)| s.id) {
            let state_name = format!(
                "State: {}{}",
                state.id,
                if let Some(label) = self.labels.get(&state) {
                    format!(" \"{}\"", label)
                } else {
                    "".into()
                }
            );

            let mut edges = vec![];
            for (word, targets) in transitions {
                for t in targets {
                    let acceptance_ids: Vec<_> = acceptance_sets
                        .iter()
                        .filter_map(|(i, s)| {
                            if s.contains(&t) {
                                Some(i.to_string())
                            } else {
                                None
                            }
                        })
                        .collect();
                    let id = if acceptance_ids.is_empty() {
                        "".into()
                    } else {
                        format!(" {{{}}}", acceptance_ids.join(" "))
                    };
                    edges.push(format!("\n  {{{}}} {}{}", word.id, t.id, id));
                }
            }

            states.push(format!("{}{}", state_name, edges.join("")));
        }

        let body = format!("--BODY--\n{}\n--END--", states.join("\n"));

        format!("{}\n{}", header, body)
    }

    pub fn to_dot(&self) -> String {
        let mut out = String::new();

        writeln!(&mut out, "digraph g {{\nmindist = 2.0").unwrap();
        for (state, transitions) in &self.states {
            for (word, targets) in transitions {
                for target in targets {
                    writeln!(
                        &mut out,
                        "\"{}\" -> {{\"{}\"}} [label = \"{}\"]",
                        self.labels[&state], self.labels[&target], word.id
                    )
                    .unwrap();
                }
            }
        }

        for (i, initial) in self.initial_states.iter().enumerate() {
            writeln!(
                &mut out,
                "init{0} [label=\"\", shape=point]\ninit{0} -> \"{1}\"",
                i, self.labels[initial]
            )
            .unwrap();
        }

        out.push('}');
        out.push('\n');
        out
    }
}

impl Buchi {
    /// Create a new empty Buchi Automata
    pub fn new() -> Self {
        Buchi {
            states: HashMap::new(),
            labels: HashMap::new(),
            accepting_sets: HashSet::new(),
            initial_states: HashSet::new(),
            size: 0,
        }
    }

    pub fn add_accepting_set(&mut self, set: impl IntoIterator<Item = State>) {
        self.accepting_sets
            .insert(BTreeSet::from_iter(set.into_iter()));
    }

    /// Generate a new state. The return value is used to construct transitions and set the initial/accepting states
    pub fn new_state(&mut self) -> State {
        let id = self.size;
        let state = State { id };
        self.size += 1;
        self.states.insert(state, HashMap::new());
        state
    }

    pub fn new_labeled_state(&mut self, label: String) -> State {
        let id = self.size;
        let state = State { id };
        self.size += 1;
        self.states.insert(state, HashMap::new());
        self.labels.insert(state, label);
        state
    }

    /// Make the provided state an initial state
    pub fn set_initial_state(&mut self, state: State) {
        self.initial_states.insert(state);
    }

    /// Make the provided states initial states
    pub fn set_initial_states(&mut self, states: &[State]) {
        self.initial_states.extend(states);
    }

    /// Add a transition from Source to Target with label Word.
    /// The Word can be any kind of string or a manually constructed Word, which should then probably be cloned
    /// since Word does not implement Copy.
    pub fn add_transition<T: Into<Word>>(&mut self, source: State, target: State, word: T) {
        // idc about the borrow checker
        let word = word.into();

        // Insert the necessary transition information
        self.states
            .entry(source)
            .or_insert(HashMap::new())
            .entry(word)
            .or_insert(HashSet::new())
            .insert(target);
    }

    /// Get a set of all states that exist in the automaton. It does not matter whether they're reachable or not.
    pub fn states(&self) -> HashSet<State> {
        self.states.keys().map(|s| s.clone()).collect()
    }

    pub fn initial_states(&self) -> &HashSet<State> {
        &self.initial_states
    }

    pub fn accepting_sets(&self) -> &HashSet<BTreeSet<State>> {
        &self.accepting_sets
    }

    pub fn label(&self, state: &State) -> Option<&str> {
        self.labels.get(state).map(String::as_str)
    }

    pub fn transitions(&self) -> Vec<Transition> {
        self.states
            .iter()
            .map(|(s, transitions)| {
                transitions.iter().map(|(label, targets)| {
                    targets.iter().map(|t| Transition {
                        from: self.labels.get(s).map(String::as_str).unwrap_or(""),
                        from_state: *s,
                        to: self.labels.get(t).map(String::as_str).unwrap_or(""),
                        to_state: *t,
                        label: &label.id,
                    })
                })
            })
            .flatten()
            .flatten()
            .collect_vec()
    }

    /// Returns a set of strongly connected components using Tarjan's algorithm
    pub fn tarjans_scc(&self) -> Vec<HashSet<State>> {
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

    fn scc_is_trivial(&self, scc: &HashSet<State>) -> bool {
        scc.len() == 1 && {
            let transitions = self.states.get(scc.iter().next().unwrap()).unwrap();
            !transitions.values().contains(scc)
        }
    }

    /// Verify that there exists no trace which satisfies the automaton
    /// If there exists a counter example give one back
    pub fn verify(&self) -> Result<(), Trace> {
        // TODO adjust this for acceptance sets instead of a single acceptance set of states
        // Gather all the final states which are contained in a non trivial SCC
        let sccs: Vec<_> = self
            .tarjans_scc()
            .into_iter()
            .filter(|c| !self.scc_is_trivial(c))
            .collect();

        // If there exists an accepting set where no state is in a non trivial SCC then there is no trace that satisfies
        for set in &self.accepting_sets {
            if set
                .iter()
                .any(|f| sccs.iter().all(|component| !component.contains(f)))
            {
                return Ok(());
            }
        }

        // If there are no accepting sets and there is no non trivial SCC then there also cannot be a trace
        if sccs.iter().all(|c| self.scc_is_trivial(c)) {
            return Ok(());
        }

        let nba = self.gnba_to_nba();
        let sccs: Vec<_> = nba
            .tarjans_scc()
            .into_iter()
            .filter(|c| !nba.scc_is_trivial(c))
            .collect();

        // We already know that the accepting set is part of an SCC, no need to check
        let accepting: HashSet<_> = nba.accepting_sets.iter().flatten().collect();

        // If there are no accepting states place an accepting state in every SCC because every infinite run is valid
        // But if the accepting sets are empty simply place an accepting state in every SCC
        let accepting = if accepting.is_empty() {
            sccs.iter().map(|scc| scc.iter().next().unwrap()).collect()
        } else {
            accepting
        };

        // If we can reach any of these accepting states we have found a counter example
        let mut visited = HashMap::new();

        for initial_state in &nba.initial_states {
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
                    let omega_trace = nba.constrained_cycle_searcher(state, scc).unwrap();

                    return Err(Trace::new(trace, omega_trace));
                }

                for transition in nba.states.get(state) {
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
                        if !visited.contains_key(successor) {
                            queue.push(successor);
                            visited.insert(successor, new_trace);
                        }
                    }
                }
            }
        }

        None
    }

    pub fn gnba_to_nba(&self) -> Self {
        // If the accepting states are empty or there's only one it doesn't matter what you do, just return the whole gnba since it's already an nba
        if self.accepting_sets.len() <= 1 {
            return self.clone();
        }
        // Clone the accepting states into a vec for deterministic ordering
        let mut nba = Buchi::new();

        for (i, accepting_set) in self.accepting_sets.iter().enumerate() {
            // Create a copy of the statespace for every accepting state and rename them to s0_0, s0_1, s0_2 etc for each iteration
            let mut new_states: HashMap<State, HashMap<Word, HashSet<State>>> = self
                .states
                .clone()
                .into_iter()
                .map(|(mut k, mut v)| {
                    // Rename the source state
                    k.id += self.size * i;
                    // Rename the target states while leaving the word the same
                    for targets in v.values_mut() {
                        *targets = targets
                            .iter()
                            .map(|s| State {
                                id: s.id + self.size * i,
                            })
                            .collect();
                    }
                    (k, v)
                })
                .collect();

            // Add new labels
            for (new, _) in &new_states {
                nba.labels.insert(
                    *new,
                    self.labels
                        .get(&State {
                            id: new.id % self.size,
                        })
                        .unwrap()
                        .clone(),
                );
            }

            // Map the transitions of the current accepting states to point towards the next one (potentially the first)
            for accepting in accepting_set {
                new_states
                    .entry(State {
                        id: accepting.id + self.size * i,
                    })
                    .and_modify(|transitions| {
                        for targets in transitions.values_mut() {
                            *targets = targets
                                .iter()
                                .map(|target| State {
                                    id: (target.id + self.size)
                                        % (self.size * self.accepting_sets.len()),
                                })
                                .collect();
                        }
                    });
            }

            nba.states.extend(new_states);

            // Make the last set accepting
            if i == self.accepting_sets.len() - 1 {
                nba.add_accepting_set(accepting_set.iter().map(|s| State {
                    id: s.id + self.size * i,
                }));
            }
        }

        for initial_state in &self.initial_states {
            nba.set_initial_state(*initial_state)
        }

        // Update the size of the nba
        nba.size += self.size * self.accepting_sets.len();

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
                .map(|s| format!("s{}", s.id))
                .collect::<Vec<_>>()
                .join(", ")
        )?;
        writeln!(
            f,
            "Accepting Sets: ({})",
            self.accepting_sets()
                .iter()
                .map(|s| format!("{{{}}}", s.iter().map(|a| format!("s{}", a.id)).join(", ")))
                .collect::<Vec<_>>()
                .join(", ")
        )?;
        writeln!(f, "Transitions:")?;
        for (s, transitions) in &self.states {
            for (word, targets) in transitions {
                for t in targets {
                    writeln!(f, "s{} --({})--> s{}", s.id, word.id, t.id)?;
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

impl<T: ToString> From<T> for Word {
    fn from(w: T) -> Self {
        Self { id: w.to_string() }
    }
}

impl Trace {
    pub fn new(words: Vec<Word>, omega_words: Vec<Word>) -> Self {
        Trace { words, omega_words }
    }
}

impl Display for Trace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if !self.words.is_empty() {
            write!(
                f,
                "{}, ",
                self.words
                    .iter()
                    .map(|w| w.id.as_str())
                    .collect::<Vec<&str>>()
                    .join(", ")
            )?;
        }
        write!(
            f,
            "({})??",
            self.omega_words
                .iter()
                .map(|w| w.id.as_str())
                .collect::<Vec<&str>>()
                .join(", ")
        )?;
        Ok(())
    }
}
