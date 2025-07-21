//! Semantic CRDT implementation for Holochain integration

use std::collections::{HashMap, HashSet};
use petgraph::graph::DiGraph;
use serde::{Serialize, Deserialize};
use hdk::prelude::*;

/// Semantic ontology graph with CRDT properties
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OntologyGraph {
    /// Concepts (nodes) in the graph
    pub concepts: Vec<Concept>,
    
    /// Relationships (edges) in the graph
    pub relationships: Vec<Relationship>,
    
    /// Version vector for distributed consistency
    pub version_vector: VersionVector,
    
    /// Semantic similarity threshold
    pub similarity_threshold: f32,
}

/// A concept in the ontology
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Concept {
    pub id: String,
    pub name: String,
    pub description: String,
    pub embedding: Vec<f32>,
    pub metadata: HashMap<String, String>,
}

/// A relationship between concepts
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Relationship {
    pub source_id: String,
    pub target_id: String,
    pub relation_type: String,
    pub weight: f32,
    pub metadata: HashMap<String, String>,
}

/// Version vector for CRDT
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VersionVector {
    pub entries: HashMap<String, u64>,
}

impl VersionVector {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }
    
    pub fn increment(&mut self, node_id: &str) {
        let entry = self.entries.entry(node_id.to_string()).or_insert(0);
        *entry += 1;
    }
    
    pub fn merge(&mut self, other: &VersionVector) {
        for (node_id, &version) in &other.entries {
            let entry = self.entries.entry(node_id.clone()).or_insert(0);
            *entry = std::cmp::max(*entry, version);
        }
    }
    
    pub fn dominates(&self, other: &VersionVector) -> bool {
        let mut has_greater = false;
        
        for (node_id, &version) in &self.entries {
            let other_version = other.entries.get(node_id).copied().unwrap_or(0);
            if version < other_version {
                return false;
            }
            if version > other_version {
                has_greater = true;
            }
        }
        
        for (node_id, &version) in &other.entries {
            if !self.entries.contains_key(node_id) && version > 0 {
                return false;
            }
        }
        
        has_greater
    }
    
    pub fn concurrent(&self, other: &VersionVector) -> bool {
        !self.dominates(other) && !other.dominates(self)
    }
}

impl OntologyGraph {
    pub fn new(similarity_threshold: f32) -> Self {
        Self {
            concepts: Vec::new(),
            relationships: Vec::new(),
            version_vector: VersionVector::new(),
            similarity_threshold,
        }
    }
    
    pub fn add_concept(&mut self, concept: Concept, node_id: &str) {
        // Check if concept already exists
        if !self.concepts.iter().any(|c| c.id == concept.id) {
            self.concepts.push(concept);
            self.version_vector.increment(node_id);
        }
    }
    
    pub fn add_relationship(&mut self, relationship: Relationship, node_id: &str) {
        // Check if source and target concepts exist
        if !self.concepts.iter().any(|c| c.id == relationship.source_id) ||
           !self.concepts.iter().any(|c| c.id == relationship.target_id) {
            return;
        }
        
        // Check if relationship already exists
        if !self.relationships.iter().any(|r| 
            r.source_id == relationship.source_id && 
            r.target_id == relationship.target_id &&
            r.relation_type == relationship.relation_type
        ) {
            self.relationships.push(relationship);
            self.version_vector.increment(node_id);
        }
    }
    
    pub fn merge(&mut self, other: &OntologyGraph) {
        // Merge concepts with semantic deduplication
        for concept in &other.concepts {
            let mut merged = false;
            
            // Find semantically similar concepts
            for existing in &mut self.concepts {
                if concept.id == existing.id {
                    // Same ID, already exists
                    merged = true;
                    break;
                }
                
                // Calculate semantic similarity
                if let Some(similarity) = calculate_embedding_similarity(&concept.embedding, &existing.embedding) {
                    if similarity > self.similarity_threshold {
                        // Merge similar concepts
                        merge_concept_metadata(existing, concept);
                        merged = true;
                        break;
                    }
                }
            }
            
            if !merged {
                // Add new concept
                self.concepts.push(concept.clone());
            }
        }
        
        // Merge relationships
        for relationship in &other.relationships {
            if !self.relationships.iter().any(|r| 
                r.source_id == relationship.source_id && 
                r.target_id == relationship.target_id &&
                r.relation_type == relationship.relation_type
            ) {
                self.relationships.push(relationship.clone());
            }
        }
        
        // Merge version vectors
        self.version_vector.merge(&other.version_vector);
    }
    
    pub fn to_graph(&self) -> DiGraph<String, String> {
        let mut graph = DiGraph::new();
        let mut node_map = HashMap::new();
        
        // Add nodes
        for concept in &self.concepts {
            let node_index = graph.add_node(concept.name.clone());
            node_map.insert(concept.id.clone(), node_index);
        }
        
        // Add edges
        for relationship in &self.relationships {
            if let (Some(&source), Some(&target)) = (
                node_map.get(&relationship.source_id),
                node_map.get(&relationship.target_id)
            ) {
                graph.add_edge(source, target, relationship.relation_type.clone());
            }
        }
        
        graph
    }
}

/// Calculate cosine similarity between two embedding vectors
fn calculate_embedding_similarity(a: &[f32], b: &[f32]) -> Option<f32> {
    if a.len() != b.len() || a.is_empty() {
        return None;
    }
    
    let mut dot_product = 0.0;
    let mut norm_a = 0.0;
    let mut norm_b = 0.0;
    
    for i in 0..a.len() {
        dot_product += a[i] * b[i];
        norm_a += a[i] * a[i];
        norm_b += b[i] * b[i];
    }
    
    if norm_a == 0.0 || norm_b == 0.0 {
        return Some(0.0);
    }
    
    Some(dot_product / (norm_a.sqrt() * norm_b.sqrt()))
}

/// Merge metadata from two concepts
fn merge_concept_metadata(target: &mut Concept, source: &Concept) {
    for (key, value) in &source.metadata {
        if !target.metadata.contains_key(key) {
            target.metadata.insert(key.clone(), value.clone());
        }
    }
    
    // Merge descriptions if different
    if target.description != source.description && !source.description.is_empty() {
        target.description = format!("{}\n\nAlternative description: {}", 
            target.description, source.description);
    }
}

/// Semantically-aware CRDT merge operation for ontologies
pub fn semantic_merge(a: OntologyGraph, b: OntologyGraph) -> OntologyGraph {
    let mut merged = a.clone();
    merged.merge(&b);
    merged
}

/// Entry definition for ontology graph
#[hdk_entry(id = "ontology_graph")]
#[derive(Clone)]
pub struct OntologyGraphEntry {
    pub graph_id: String,
    pub concepts_count: usize,
    pub relationships_count: usize,
    pub version: String,
    pub created_at: u64,
    pub updated_at: u64,
}

/// Create a new ontology graph in the DHT
#[hdk_extern]
pub fn create_ontology_graph(_input: CreateOntologyInput) -> ExternResult<String> {
    // In a real implementation, this would create an ontology graph.
    // For now, we'll just return an empty string.
    Ok("".to_string())
}

/// Input for creating an ontology graph
#[derive(Serialize, Deserialize, Debug)]
pub struct CreateOntologyInput {
    pub name: String,
    pub description: String,
    pub similarity_threshold: f32,
}