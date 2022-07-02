use std::{
    collections::{HashMap, HashSet, VecDeque},
    fmt::Display,
};

use colored::Colorize;
use itertools::Itertools;
use petgraph::{graph::NodeIndex, EdgeDirection::Incoming};

use crate::{Graph, Owner, Solution};

struct MeasureFactory {
    tuple_size: usize,
    max_measure: Measure,
    is_odd: bool,
}

impl MeasureFactory {
    fn new(graph: &Graph, player: Owner) -> Self {
        let max_priority = graph.highest_priority().unwrap();
        let tuple_size = if max_priority % 2 == 0 {
            match player {
                Owner::Even => max_priority / 2,
                Owner::Odd => max_priority / 2 + 1,
            }
        } else {
            max_priority / 2 + 1
        };

        let max_tuple = (0..tuple_size)
            .rev()
            .map(|i| {
                let priority = match player {
                    Owner::Even => i * 2 + 1,
                    Owner::Odd => i * 2,
                };
                Some(
                    graph
                        .inner
                        .node_weights()
                        .filter(|v| v.priority == priority)
                        .count(),
                )
            })
            .collect_vec();

        let is_odd = player == Owner::Odd;
        MeasureFactory {
            tuple_size,
            is_odd,
            max_measure: Measure {
                tuple: max_tuple,
                is_odd,
                is_max: true,
            },
        }
    }
    fn zero_measure(&self) -> Measure {
        let tuple = vec![Some(0); self.tuple_size];
        Measure {
            tuple,
            is_odd: self.is_odd,
            is_max: false,
        }
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
struct Measure {
    tuple: Vec<Option<usize>>,
    is_odd: bool,
    is_max: bool,
}

impl Measure {
    fn priority_to_index(&self, p: usize) -> usize {
        let i = p / 2 + 1;
        self.tuple.len().saturating_sub(i)
    }

    fn prune(&self, p: usize) -> Measure {
        let mut new_tuple = self.tuple.clone();
        // If the measure is about odd priorities start at 1, otherwise 0
        let mut r_i = if self.is_odd { 0 } else { 1 };
        while r_i < p {
            new_tuple[self.priority_to_index(r_i)] = None;
            r_i += 2;
        }

        Measure {
            tuple: new_tuple,
            is_odd: self.is_odd,
            is_max: false,
        }
    }
}

impl Display for Measure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_max {
            write!(f, "<T>")
        } else {
            write!(
                f,
                "<{}>",
                self.tuple
                    .iter()
                    .map(|m| if let Some(m) = m {
                        m.to_string()
                    } else {
                        "-".to_string()
                    })
                    .join(", ")
            )
        }
    }
}

impl Graph {
    pub fn spm(&self) -> Solution {
        log::info!("solving with SPM");
        if self.inner.node_count() == 0 {
            return Solution::empty();
        }

        let (w_0, w_1, s_0) = self.progress_measure(Owner::Even);
        let s_1 = if w_1.is_empty() {
            log::info!("odd has no winning vertices, no need to recompute");
            HashMap::new()
        } else {
            log::info!(
                "odd has a winning region, recomputing progress measure to determine strategy"
            );
            self.progress_measure(Owner::Odd).2
        };

        self.construct_solution(w_0, w_1, s_0, s_1)
    }

    fn progress_measure(
        &self,
        player: Owner,
    ) -> (
        HashSet<NodeIndex>,
        HashSet<NodeIndex>,
        HashMap<NodeIndex, NodeIndex>,
    ) {
        log::info!("executing small progress measure for player {}", player);
        let measure_factory = MeasureFactory::new(self, player);

        log::debug!(
            "the maximum measure is <{}>",
            measure_factory
                .max_measure
                .tuple
                .iter()
                .map(|e| e.unwrap())
                .join(", ")
        );

        let mut measures: HashMap<_, _> = self
            .inner
            .node_indices()
            .map(|v| (v, measure_factory.zero_measure()))
            .collect();

        let mut q: VecDeque<_> = self
            .inner
            .node_indices()
            .filter(|v| Owner::from_usize(self.inner[*v].priority) != player)
            .collect();

        while let Some(v) = q.pop_front() {
            let lift = self.lift(player, &measures, v, &measure_factory.max_measure);
            if measures[&v] < lift {
                log::debug!("{} {} to {}", "lifting".red(), self.debug_vertice(v), lift);
                measures.insert(v, lift);
                for n in self.inner.neighbors_directed(v, Incoming) {
                    if !q.contains(&n) {
                        q.push_back(n);
                    }
                }
            }
        }

        log::debug!(
            "final measures: {}",
            measures
                .iter()
                .map(|(k, v)| format!("{}: {}", self.debug_vertice(*k), v))
                .join(", ")
        );
        let (w_alpha, w_beta): (HashSet<_>, HashSet<_>) = self
            .inner
            .node_indices()
            .partition(|v| !measures[&v].is_max);

        log::debug!("w_alpha: {}", self.debug(&w_alpha));
        log::debug!("w_beta: {}", self.debug(&w_beta));

        let sigma_alpha: HashMap<_, _> = w_alpha
            .iter()
            .filter(|v| self.inner[**v].owner == player)
            .filter_map(|v| {
                let mut targets = self.inner.neighbors(*v).filter(|n| {
                    measures[&v]
                        == prog(
                            &measures[&n],
                            self.inner[*v].priority,
                            player,
                            &measure_factory.max_measure,
                        )
                });
                if let Some(t) = targets.next() {
                    Some((*v, t))
                } else {
                    None
                }
            })
            .collect();

        log::debug!(
            "{} for player {} {{{}}}",
            "strategy calculated".bright_green(),
            player,
            sigma_alpha
                .iter()
                .map(|(k, v)| format!("{} -> {}", self.debug_vertice(*k), self.debug_vertice(*v)))
                .join(", ")
        );

        (w_alpha, w_beta, sigma_alpha)
    }

    fn lift(
        &self,
        player: Owner,
        measures: &HashMap<NodeIndex, Measure>,
        vertex: NodeIndex,
        max_measure: &Measure,
    ) -> Measure {
        if self.inner[vertex].owner == player {
            self.inner
                .neighbors(vertex)
                .map(|n| {
                    prog(
                        &measures[&n],
                        self.inner[vertex].priority,
                        player,
                        max_measure,
                    )
                })
                .min()
                .expect("Could not find a minimum")
        } else {
            self.inner
                .neighbors(vertex)
                .map(|n| {
                    prog(
                        &measures[&n],
                        self.inner[vertex].priority,
                        player,
                        max_measure,
                    )
                })
                .max()
                .expect("Could not find a maximum")
        }
    }
}

fn prog(measure: &Measure, p: usize, player: Owner, max_measure: &Measure) -> Measure {
    if measure == max_measure {
        return max_measure.clone();
    }

    let mut m = measure.prune(p);
    if Owner::from_usize(p) != player {
        for (elem, max_elem) in m.tuple.iter_mut().zip(&max_measure.tuple).rev() {
            let max_val = max_elem.expect("inconsistent maximal measure");
            if let Some(e) = elem {
                if *e == max_val {
                    // if the element is the max value set it to 0 and continue
                    *elem = Some(0);
                } else {
                    // if we can increment do that and stop modifying
                    *elem = Some(*e + 1);
                    break;
                }
            } else {
                *elem = Some(0);
            }
        }

        // If everything is 0, then the only value larger than the current measure is maximal
        if m.tuple.iter().all(|e| *e == Some(0)) {
            m = max_measure.clone()
        }
    }

    for e in m.tuple.iter_mut() {
        if let None = e {
            *e = Some(0)
        }
    }

    m
}
