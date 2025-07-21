use hdk::prelude::*;
use hnsw::{Hnsw, Searcher};
use petgraph::graph::DiGraph;
use std::collections::HashMap;

pub struct HnswWrapper {
    hnsw: Hnsw<f32, usize>,
    searcher: Searcher,
    data: HashMap<usize, Vec<f32>>,
}

impl HnswWrapper {
    pub fn new() -> Self {
        Self {
            hnsw: Hnsw::new(16, 100, 32, 200),
            searcher: Searcher::new(),
            data: HashMap::new(),
        }
    }

    pub fn add(&mut self, id: usize, vector: &[f32]) {
        self.hnsw.insert(vector, id);
        self.data.insert(id, vector.to_vec());
    }

    pub fn search(&self, vector: &[f32], k: usize) -> Vec<(usize, f32)> {
        let mut neighbors = vec![];
        self.hnsw.search(vector, k, &mut self.searcher, |neighbor_id| {
            let neighbor_vector = &self.data[&neighbor_id];
            let distance = dot_product(vector, neighbor_vector);
            neighbors.push((neighbor_id, distance));
        });
        neighbors
    }
}

fn dot_product(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b).map(|(x, y)| x * y).sum()
}
pub struct KnowledgeGraph {
    graph: DiGraph<String, String>,
    node_map: HashMap<String, petgraph::graph::NodeIndex>,
}

impl KnowledgeGraph {
    pub fn new() -> Self {
        Self {
            graph: DiGraph::new(),
            node_map: HashMap::new(),
        }
    }

    pub fn add_concept(&mut self, id: &str, name: &str) {
        let node_index = self.graph.add_node(name.to_string());
        self.node_map.insert(id.to_string(), node_index);
    }

    pub fn add_relationship(&mut self, from: &str, to: &str, label: &str) {
        if let (Some(&from_index), Some(&to_index)) =
            (self.node_map.get(from), self.node_map.get(to))
        {
            self.graph.add_edge(from_index, to_index, label.to_string());
        }
    }
}

#[hdk_extern]
pub fn init(_: ()) -> ExternResult<InitCallbackResult> {
    Ok(InitCallbackResult::Pass)
}
