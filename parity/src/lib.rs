use std::{collections::BTreeSet, ffi::OsStr};

use petgraph::graph::{DiGraph, NodeIndex};
// The main data structure is a Graph
// Each vertex contains information:
// - What is the priority (a number from 0 to n)
// - What is the
struct Priority {
    _label: String,
    p: usize,
}

pub struct Graph {
    inner: DiGraph<Priority, ()>,
}

impl Graph {
    pub fn parse_file(path: &OsStr) -> Self {
        todo!()
    }

    fn new() -> Self {
        Graph {
            inner: DiGraph::new(),
        }
    }

    fn highest_priority(&self) -> Option<usize> {
        self.inner.node_weights().map(|n| n.p).max()
    }

    fn winner(&self, v: NodeIndex, z: &BTreeSet<NodeIndex>) -> usize {
        let p = self
            .inner
            .node_weight(v)
            .expect("Could not find node with given weight")
            .p
            % 2;
        if !z.contains(&v) {
            p
        } else {
            1 - p
        }
    }
    fn onestep(&self, v: NodeIndex, z: &BTreeSet<NodeIndex>) -> usize {
        let p = self
            .inner
            .node_weight(v)
            .expect("Could not find node with given weight")
            .p;

        if p % 2 == 0 {
            if self
                .inner
                .neighbors(v)
                .into_iter()
                .any(|n| self.winner(n, z) == 0)
            {
                0
            } else {
                1
            }
        } else {
            if self
                .inner
                .neighbors(v)
                .into_iter()
                .any(|n| self.winner(n, z) == 1)
            {
                1
            } else {
                0
            }
        }
    }

    pub fn fpi(&self) -> (BTreeSet<NodeIndex>, BTreeSet<NodeIndex>) {
        let mut z = BTreeSet::new();
        let mut p = 0;
        let max_priority = self
            .highest_priority()
            .expect("Graph was empty, cannot determine highest priority");

        while p <= max_priority {
            let parity = p % 2;
            let y: BTreeSet<_> = self
                .inner
                .node_indices()
                .into_iter()
                .filter(|v| z.contains(v))
                .filter(|v| self.onestep(*v, &z) != parity)
                .collect();
            if y.is_empty() {
                z = z
                    .intersection(&y)
                    .cloned()
                    .filter(|v| {
                        self.inner
                            .node_weight(*v)
                            .expect("Could not find node weight for given index")
                            .p
                            >= p
                    })
                    .collect();
                p = 0;
            } else {
                p += 1;
            }
        }
        self.inner
            .node_indices()
            .into_iter()
            .partition(|v| self.winner(*v, &z) == 0)
    }
}
