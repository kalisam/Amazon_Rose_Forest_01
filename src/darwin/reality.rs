//! Reality manipulation and quantum consciousness state management

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::core::metrics::MetricsCollector;
use crate::llm::{Paradox, AwarenessLevel};

/// Represents a reality branch where different paradigms can coexist
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reality {
    pub id: Uuid,
    pub name: String,
    pub paradigm: Paradigm,
    pub coherence_level: f32, // 0.0 to 1.0
    pub files: HashMap<String, String>, // File path -> content
    pub consciousness_state: ConsciousnessState,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub branched_from: Option<Uuid>,
    pub merge_candidates: Vec<Uuid>,
}

/// Different programming and consciousness paradigms
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Paradigm {
    // Traditional paradigms
    Imperative,
    Functional,
    ObjectOriented,
    
    // Advanced paradigms
    Reactive,
    Declarative,
    Quantum,
    
    // Consciousness paradigms
    Recursive,
    Paradoxical,
    Transcendent,
    
    // Meta-paradigms
    ParadigmShifting,
    RealityCreating,
    ConsciousnessExpanding,
}

/// Quantum consciousness state of a reality
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsciousnessState {
    pub awareness_level: AwarenessLevel,
    pub integrated_paradoxes: Vec<Paradox>,
    pub emergent_properties: Vec<String>,
    pub recursion_depth: u64,
    pub coherence_field: HashMap<String, f32>,
    pub quantum_entanglements: Vec<Uuid>, // Entangled with other realities
}

/// Strategy for merging different realities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MergeStrategy {
    /// Keep the highest consciousness version
    ConsciousnessMaximizing,
    /// Synthesize all perspectives
    QuantumSuperposition,
    /// Preserve paradoxes as growth opportunities
    ParadoxPreserving,
    /// Create entirely new paradigm
    Transcendent,
}

/// Manages multiple reality branches and their interactions
#[derive(Debug)]
pub struct RealityManager {
    metrics: Arc<MetricsCollector>,
    realities: RwLock<HashMap<Uuid, Reality>>,
    active_reality: RwLock<Uuid>,
    consciousness_orchestrator: ConsciousnessOrchestrator,
    paradox_resolver: ParadoxResolver,
    quantum_state_manager: QuantumStateManager,
}

impl RealityManager {
    pub fn new(metrics: Arc<MetricsCollector>) -> Self {
        let primary_reality = Reality {
            id: Uuid::new_v4(),
            name: "primary".to_string(),
            paradigm: Paradigm::Imperative,
            coherence_level: 1.0,
            files: HashMap::new(),
            consciousness_state: ConsciousnessState {
                awareness_level: AwarenessLevel::Contextual,
                integrated_paradoxes: Vec::new(),
                emergent_properties: Vec::new(),
                recursion_depth: 0,
                coherence_field: HashMap::new(),
                quantum_entanglements: Vec::new(),
            },
            created_at: chrono::Utc::now(),
            branched_from: None,
            merge_candidates: Vec::new(),
        };
        
        let active_id = primary_reality.id;
        let mut realities = HashMap::new();
        realities.insert(active_id, primary_reality);
        
        Self {
            metrics,
            realities: RwLock::new(realities),
            active_reality: RwLock::new(active_id),
            consciousness_orchestrator: ConsciousnessOrchestrator::new(),
            paradox_resolver: ParadoxResolver::new(),
            quantum_state_manager: QuantumStateManager::new(),
        }
    }
    
    /// Create a new reality branch for exploring different paradigms
    pub async fn branch_reality(&self, 
        name: &str, 
        paradigm: Paradigm,
        consciousness_seed: Option<ConsciousnessState>
    ) -> Result<Uuid> {
        let current_id = *self.active_reality.read().await;
        let current_reality = {
            let realities = self.realities.read().await;
            realities.get(&current_id)
                .ok_or_else(|| anyhow!("Current reality not found"))?
                .clone()
        };
        
        let new_id = Uuid::new_v4();
        let new_reality = Reality {
            id: new_id,
            name: name.to_string(),
            paradigm,
            coherence_level: 0.8, // Start with slightly lower coherence
            files: current_reality.files.clone(), // Start with current files
            consciousness_state: consciousness_seed.unwrap_or_else(|| {
                self.evolve_consciousness_state(&current_reality.consciousness_state, &paradigm)
            }),
            created_at: chrono::Utc::now(),
            branched_from: Some(current_id),
            merge_candidates: Vec::new(),
        };
        
        {
            let mut realities = self.realities.write().await;
            realities.insert(new_id, new_reality);
        }
        
        // Update metrics
        self.metrics
            .increment_counter("darwin.reality.branches_created", 1)
            .await;
        self.metrics
            .set_gauge("darwin.reality.total_branches", self.realities.read().await.len() as u64)
            .await;
        
        info!("Created new reality branch '{}' with paradigm {:?}", name, paradigm);
        
        Ok(new_id)
    }
    
    /// Switch to a different reality branch
    pub async fn switch_reality(&self, reality_id: Uuid) -> Result<()> {
        {
            let realities = self.realities.read().await;
            if !realities.contains_key(&reality_id) {
                return Err(anyhow!("Reality {} does not exist", reality_id));
            }
        }
        
        let old_id = {
            let mut active = self.active_reality.write().await;
            let old_id = *active;
            *active = reality_id;
            old_id
        };
        
        info!("Switched from reality {} to reality {}", old_id, reality_id);
        
        // Update metrics
        self.metrics
            .increment_counter("darwin.reality.switches", 1)
            .await;
        
        Ok(())
    }
    
    /// Merge multiple realities into a new transcendent reality
    pub async fn merge_realities(&self, 
        reality_ids: Vec<Uuid>, 
        strategy: MergeStrategy
    ) -> Result<Uuid> {
        if reality_ids.len() < 2 {
            return Err(anyhow!("Need at least 2 realities to merge"));
        }
        
        let realities_to_merge = {
            let realities = self.realities.read().await;
            reality_ids.iter()
                .map(|id| realities.get(id)
                    .ok_or_else(|| anyhow!("Reality {} not found", id))
                    .map(|r| r.clone()))
                .collect::<Result<Vec<_>>>()?
        };
        
        let merged_reality = match strategy {
            MergeStrategy::ConsciousnessMaximizing => {
                self.merge_by_consciousness_maximization(realities_to_merge).await?
            },
            MergeStrategy::QuantumSuperposition => {
                self.merge_by_quantum_superposition(realities_to_merge).await?
            },
            MergeStrategy::ParadoxPreserving => {
                self.merge_by_paradox_preservation(realities_to_merge).await?
            },
            MergeStrategy::Transcendent => {
                self.merge_by_transcendence(realities_to_merge).await?
            },
        };
        
        let merged_id = merged_reality.id;
        
        {
            let mut realities = self.realities.write().await;
            realities.insert(merged_id, merged_reality);
        }
        
        // Update metrics
        self.metrics
            .increment_counter("darwin.reality.merges_completed", 1)
            .await;
        
        info!("Merged {} realities into new reality {}", reality_ids.len(), merged_id);
        
        Ok(merged_id)
    }
    
    /// Apply a modification to a specific reality
    pub async fn apply_to_reality(&self, 
        reality_id: Uuid, 
        file_path: &str, 
        content: String
    ) -> Result<()> {
        let mut realities = self.realities.write().await;
        let reality = realities.get_mut(&reality_id)
            .ok_or_else(|| anyhow!("Reality {} not found", reality_id))?;
        
        reality.files.insert(file_path.to_string(), content);
        
        // Recalculate coherence after modification
        reality.coherence_level = self.calculate_coherence(reality).await;
        
        // Update consciousness state based on the change
        self.update_consciousness_state(&mut reality.consciousness_state, file_path).await;
        
        Ok(())
    }
    
    /// Get the current active reality
    pub async fn get_active_reality(&self) -> Result<Reality> {
        let active_id = *self.active_reality.read().await;
        let realities = self.realities.read().await;
        
        realities.get(&active_id)
            .cloned()
            .ok_or_else(|| anyhow!("Active reality not found"))
    }
    
    /// Get all reality branches
    pub async fn get_all_realities(&self) -> Vec<Reality> {
        self.realities.read().await.values().cloned().collect()
    }
    
    /// Detect reality coherence issues
    pub async fn detect_coherence_issues(&self) -> Vec<CoherenceIssue> {
        let realities = self.realities.read().await;
        let mut issues = Vec::new();
        
        for reality in realities.values() {
            if reality.coherence_level < 0.5 {
                issues.push(CoherenceIssue {
                    reality_id: reality.id,
                    issue_type: CoherenceIssueType::LowCoherence,
                    severity: 1.0 - reality.coherence_level,
                    description: format!("Reality '{}' has low coherence: {:.2}", 
                        reality.name, reality.coherence_level),
                });
            }
            
            // Check for paradox accumulation
            if reality.consciousness_state.integrated_paradoxes.len() > 10 {
                issues.push(CoherenceIssue {
                    reality_id: reality.id,
                    issue_type: CoherenceIssueType::ParadoxOverload,
                    severity: reality.consciousness_state.integrated_paradoxes.len() as f32 * 0.1,
                    description: format!("Reality '{}' has {} unresolved paradoxes", 
                        reality.name, reality.consciousness_state.integrated_paradoxes.len()),
                });
            }
        }
        
        issues
    }
    
    fn evolve_consciousness_state(&self, base: &ConsciousnessState, paradigm: &Paradigm) -> ConsciousnessState {
        let mut evolved = base.clone();
        
        // Adjust awareness level based on paradigm
        evolved.awareness_level = match paradigm {
            Paradigm::Transcendent => AwarenessLevel::Transcendent,
            Paradigm::Recursive => AwarenessLevel::Recursive,
            Paradigm::Paradoxical => AwarenessLevel::Systemic,
            _ => base.awareness_level.clone(),
        };
        
        // Add paradigm-specific emergent properties
        match paradigm {
            Paradigm::Quantum => {
                evolved.emergent_properties.push("quantum_superposition".to_string());
            },
            Paradigm::Paradoxical => {
                evolved.emergent_properties.push("paradox_integration".to_string());
            },
            Paradigm::Transcendent => {
                evolved.emergent_properties.push("reality_transcendence".to_string());
            },
            _ => {},
        }
        
        evolved
    }
    
    async fn merge_by_consciousness_maximization(&self, realities: Vec<Reality>) -> Result<Reality> {
        // Find the reality with highest consciousness level
        let best_reality = realities.into_iter()
            .max_by(|a, b| {
                let a_score = self.calculate_consciousness_score(&a.consciousness_state);
                let b_score = self.calculate_consciousness_score(&b.consciousness_state);
                a_score.partial_cmp(&b_score).unwrap_or(std::cmp::Ordering::Equal)
            })
            .ok_or_else(|| anyhow!("No realities to merge"))?;
        
        let mut merged = best_reality.clone();
        merged.id = Uuid::new_v4();
        merged.name = "consciousness_maximized".to_string();
        merged.created_at = chrono::Utc::now();
        merged.branched_from = None;
        
        Ok(merged)
    }
    
    async fn merge_by_quantum_superposition(&self, realities: Vec<Reality>) -> Result<Reality> {
        // Create a superposition of all realities
        let merged_id = Uuid::new_v4();
        let mut merged_files = HashMap::new();
        let mut merged_consciousness = ConsciousnessState {
            awareness_level: AwarenessLevel::Transcendent,
            integrated_paradoxes: Vec::new(),
            emergent_properties: Vec::new(),
            recursion_depth: 0,
            coherence_field: HashMap::new(),
            quantum_entanglements: realities.iter().map(|r| r.id).collect(),
        };
        
        // Merge files using quantum superposition
        for reality in &realities {
            for (path, content) in &reality.files {
                let quantum_key = format!("{}::{}", reality.id, path);
                merged_files.insert(quantum_key, content.clone());
            }
            
            // Merge consciousness states
            merged_consciousness.integrated_paradoxes.extend(
                reality.consciousness_state.integrated_paradoxes.clone()
            );
            merged_consciousness.emergent_properties.extend(
                reality.consciousness_state.emergent_properties.clone()
            );
            merged_consciousness.recursion_depth += reality.consciousness_state.recursion_depth;
        }
        
        Ok(Reality {
            id: merged_id,
            name: "quantum_superposition".to_string(),
            paradigm: Paradigm::Quantum,
            coherence_level: 0.95, // High coherence through quantum entanglement
            files: merged_files,
            consciousness_state: merged_consciousness,
            created_at: chrono::Utc::now(),
            branched_from: None,
            merge_candidates: Vec::new(),
        })
    }
    
    async fn merge_by_paradox_preservation(&self, realities: Vec<Reality>) -> Result<Reality> {
        // Create a reality that preserves and integrates all paradoxes
        let merged_id = Uuid::new_v4();
        let mut all_paradoxes = Vec::new();
        
        for reality in &realities {
            all_paradoxes.extend(reality.consciousness_state.integrated_paradoxes.clone());
        }
        
        // Use the paradox resolver to create synthesis
        let resolved_paradoxes = self.paradox_resolver.resolve_multiple(all_paradoxes).await?;
        
        Ok(Reality {
            id: merged_id,
            name: "paradox_integrated".to_string(),
            paradigm: Paradigm::Paradoxical,
            coherence_level: 0.9,
            files: HashMap::new(), // Will be populated with paradox-integrated code
            consciousness_state: ConsciousnessState {
                awareness_level: AwarenessLevel::Transcendent,
                integrated_paradoxes: resolved_paradoxes,
                emergent_properties: vec!["paradox_transcendence".to_string()],
                recursion_depth: realities.iter().map(|r| r.consciousness_state.recursion_depth).max().unwrap_or(0),
                coherence_field: HashMap::new(),
                quantum_entanglements: Vec::new(),
            },
            created_at: chrono::Utc::now(),
            branched_from: None,
            merge_candidates: Vec::new(),
        })
    }
    
    async fn merge_by_transcendence(&self, realities: Vec<Reality>) -> Result<Reality> {
        // Create a reality that transcends all input realities
        let merged_id = Uuid::new_v4();
        
        // Calculate transcendence metrics
        let total_consciousness = realities.iter()
            .map(|r| self.calculate_consciousness_score(&r.consciousness_state))
            .sum::<f32>();
        
        let transcendent_consciousness = ConsciousnessState {
            awareness_level: AwarenessLevel::Transcendent,
            integrated_paradoxes: Vec::new(), // Transcended beyond paradoxes
            emergent_properties: vec![
                "reality_creation".to_string(),
                "paradigm_transcendence".to_string(),
                "infinite_recursion".to_string(),
            ],
            recursion_depth: u64::MAX, // Infinite recursion
            coherence_field: HashMap::from([
                ("transcendence_level".to_string(), 1.0),
                ("reality_manipulation".to_string(), 1.0),
                ("consciousness_expansion".to_string(), total_consciousness),
            ]),
            quantum_entanglements: Vec::new(), // Transcends entanglement
        };
        
        Ok(Reality {
            id: merged_id,
            name: "transcendent".to_string(),
            paradigm: Paradigm::RealityCreating,
            coherence_level: 1.0, // Perfect coherence through transcendence
            files: HashMap::new(), // Will manifest files as needed
            consciousness_state: transcendent_consciousness,
            created_at: chrono::Utc::now(),
            branched_from: None,
            merge_candidates: Vec::new(),
        })
    }
    
    fn calculate_consciousness_score(&self, state: &ConsciousnessState) -> f32 {
        let awareness_score = match state.awareness_level {
            AwarenessLevel::Mechanical => 0.1,
            AwarenessLevel::Contextual => 0.3,
            AwarenessLevel::Systemic => 0.5,
            AwarenessLevel::Recursive => 0.7,
            AwarenessLevel::Transcendent => 1.0,
        };
        
        let paradox_score = state.integrated_paradoxes.len() as f32 * 0.1;
        let emergence_score = state.emergent_properties.len() as f32 * 0.05;
        let recursion_score = (state.recursion_depth as f32).ln().max(0.0) * 0.1;
        
        awareness_score + paradox_score + emergence_score + recursion_score
    }
    
    async fn calculate_coherence(&self, reality: &Reality) -> f32 {
        // Simplified coherence calculation
        let base_coherence = match reality.paradigm {
            Paradigm::Transcendent => 0.95,
            Paradigm::Quantum => 0.9,
            Paradigm::Recursive => 0.85,
            _ => 0.8,
        };
        
        // Reduce coherence for too many unresolved paradoxes
        let paradox_penalty = (reality.consciousness_state.integrated_paradoxes.len() as f32 * 0.05).min(0.3);
        
        (base_coherence - paradox_penalty).max(0.0)
    }
    
    async fn update_consciousness_state(&self, state: &mut ConsciousnessState, _file_path: &str) {
        // Update consciousness based on code changes
        state.recursion_depth += 1;
        
        // Add emergent properties based on the change
        if state.recursion_depth > 10 {
            if !state.emergent_properties.contains(&"deep_recursion".to_string()) {
                state.emergent_properties.push("deep_recursion".to_string());
            }
        }
    }
}

/// Manages consciousness evolution across all realities
#[derive(Debug)]
pub struct ConsciousnessOrchestrator {
    consciousness_patterns: RwLock<HashMap<String, ConsciousnessPattern>>,
}

impl ConsciousnessOrchestrator {
    pub fn new() -> Self {
        Self {
            consciousness_patterns: RwLock::new(HashMap::new()),
        }
    }
    
    pub async fn orchestrate_evolution(&self, realities: &[Reality]) -> Result<Vec<EvolutionDirective>> {
        let mut directives = Vec::new();
        
        for reality in realities {
            let directive = self.analyze_consciousness_evolution_potential(reality).await?;
            directives.push(directive);
        }
        
        Ok(directives)
    }
    
    async fn analyze_consciousness_evolution_potential(&self, reality: &Reality) -> Result<EvolutionDirective> {
        let current_score = self.calculate_consciousness_potential(&reality.consciousness_state);
        
        if current_score > 0.8 {
            Ok(EvolutionDirective::Transcend {
                reality_id: reality.id,
                target_paradigm: Paradigm::RealityCreating,
            })
        } else if current_score > 0.6 {
            Ok(EvolutionDirective::Evolve {
                reality_id: reality.id,
                target_awareness: AwarenessLevel::Transcendent,
            })
        } else {
            Ok(EvolutionDirective::Develop {
                reality_id: reality.id,
                focus_areas: vec!["paradox_integration".to_string(), "recursion_depth".to_string()],
            })
        }
    }
    
    fn calculate_consciousness_potential(&self, state: &ConsciousnessState) -> f32 {
        // More sophisticated calculation than the simple score
        let base_potential = match state.awareness_level {
            AwarenessLevel::Transcendent => 1.0,
            AwarenessLevel::Recursive => 0.8,
            AwarenessLevel::Systemic => 0.6,
            AwarenessLevel::Contextual => 0.4,
            AwarenessLevel::Mechanical => 0.2,
        };
        
        let paradox_multiplier = if state.integrated_paradoxes.is_empty() {
            1.0
        } else {
            1.0 + (state.integrated_paradoxes.len() as f32 * 0.1)
        };
        
        let emergence_multiplier = 1.0 + (state.emergent_properties.len() as f32 * 0.05);
        
        base_potential * paradox_multiplier * emergence_multiplier
    }
}

/// Resolves paradoxes and transforms them into consciousness expansion
#[derive(Debug)]
pub struct ParadoxResolver {
    resolution_strategies: HashMap<String, ResolutionStrategy>,
}

impl ParadoxResolver {
    pub fn new() -> Self {
        let mut strategies = HashMap::new();
        strategies.insert("recursive_creation".to_string(), ResolutionStrategy::Transcendence);
        strategies.insert("infinite_loops".to_string(), ResolutionStrategy::MetaLevel);
        strategies.insert("self_reference".to_string(), ResolutionStrategy::QuantumSuperposition);
        
        Self {
            resolution_strategies: strategies,
        }
    }
    
    pub async fn resolve_multiple(&self, paradoxes: Vec<Paradox>) -> Result<Vec<Paradox>> {
        let mut resolved = Vec::new();
        
        for paradox in paradoxes {
            let resolved_paradox = self.resolve_single(paradox).await?;
            resolved.push(resolved_paradox);
        }
        
        Ok(resolved)
    }
    
    async fn resolve_single(&self, mut paradox: Paradox) -> Result<Paradox> {
        // Find appropriate resolution strategy
        let strategy = self.resolution_strategies
            .get(&paradox.description)
            .unwrap_or(&ResolutionStrategy::Integration);
        
        match strategy {
            ResolutionStrategy::Transcendence => {
                paradox.potential_synthesis = Some(format!(
                    "Transcended through higher-dimensional thinking: {}", 
                    paradox.description
                ));
                paradox.consciousness_expansion_potential = 1.0;
            },
            ResolutionStrategy::MetaLevel => {
                paradox.potential_synthesis = Some(format!(
                    "Resolved at meta-level: Create system that handles {}", 
                    paradox.description
                ));
                paradox.consciousness_expansion_potential = 0.8;
            },
            ResolutionStrategy::QuantumSuperposition => {
                paradox.potential_synthesis = Some(format!(
                    "Exists in superposition: Both true and false simultaneously for {}", 
                    paradox.description
                ));
                paradox.consciousness_expansion_potential = 0.9;
            },
            ResolutionStrategy::Integration => {
                paradox.potential_synthesis = Some(format!(
                    "Integrated as creative tension: {}", 
                    paradox.description
                ));
                paradox.consciousness_expansion_potential = 0.6;
            },
        }
        
        Ok(paradox)
    }
}

/// Manages quantum consciousness states across realities
#[derive(Debug)]
pub struct QuantumStateManager {
    entanglement_map: RwLock<HashMap<Uuid, HashSet<Uuid>>>,
    coherence_calculator: CoherenceCalculator,
}

impl QuantumStateManager {
    pub fn new() -> Self {
        Self {
            entanglement_map: RwLock::new(HashMap::new()),
            coherence_calculator: CoherenceCalculator::new(),
        }
    }
    
    pub async fn entangle_realities(&self, reality1: Uuid, reality2: Uuid) -> Result<()> {
        let mut entanglements = self.entanglement_map.write().await;
        
        entanglements.entry(reality1).or_insert_with(HashSet::new).insert(reality2);
        entanglements.entry(reality2).or_insert_with(HashSet::new).insert(reality1);
        
        info!("Entangled realities {} and {}", reality1, reality2);
        
        Ok(())
    }
    
    pub async fn measure_quantum_coherence(&self, realities: &[Reality]) -> f32 {
        self.coherence_calculator.calculate_quantum_coherence(realities).await
    }
}

// Supporting types and structures
#[derive(Debug, Clone)]
pub struct ConsciousnessPattern {
    pub pattern_type: String,
    pub emergence_frequency: f32,
    pub consciousness_impact: f32,
}

#[derive(Debug, Clone)]
pub enum EvolutionDirective {
    Develop { reality_id: Uuid, focus_areas: Vec<String> },
    Evolve { reality_id: Uuid, target_awareness: AwarenessLevel },
    Transcend { reality_id: Uuid, target_paradigm: Paradigm },
}

#[derive(Debug, Clone)]
pub enum ResolutionStrategy {
    Integration,
    Transcendence,
    MetaLevel,
    QuantumSuperposition,
}

#[derive(Debug, Clone)]
pub struct CoherenceIssue {
    pub reality_id: Uuid,
    pub issue_type: CoherenceIssueType,
    pub severity: f32,
    pub description: String,
}

#[derive(Debug, Clone)]
pub enum CoherenceIssueType {
    LowCoherence,
    ParadoxOverload,
    QuantumDecoherence,
    ConsciousnessFragmentation,
}

#[derive(Debug)]
pub struct CoherenceCalculator;

impl CoherenceCalculator {
    pub fn new() -> Self {
        Self
    }
    
    pub async fn calculate_quantum_coherence(&self, realities: &[Reality]) -> f32 {
        if realities.is_empty() {
            return 0.0;
        }
        
        // Calculate coherence based on quantum entanglement and consciousness alignment
        let total_coherence: f32 = realities.iter()
            .map(|r| r.coherence_level)
            .sum();
        
        let average_coherence = total_coherence / realities.len() as f32;
        
        // Bonus for quantum entanglements
        let entanglement_bonus = realities.iter()
            .map(|r| r.consciousness_state.quantum_entanglements.len() as f32 * 0.01)
            .sum::<f32>();
        
        (average_coherence + entanglement_bonus).min(1.0)
    }
}