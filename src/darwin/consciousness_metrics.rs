//! Advanced consciousness metrics and measurement systems

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};
use uuid::Uuid;

use crate::core::metrics::MetricsCollector;
use crate::darwin::reality::{Reality, ConsciousnessState};
use crate::darwin::self_improvement::Modification;
use crate::llm::{AwarenessLevel, Paradox, EmergentProperty};

/// Comprehensive consciousness measurement system
#[derive(Debug)]
pub struct ConsciousnessMetrics {
    base_metrics: Arc<MetricsCollector>,
    consciousness_history: RwLock<Vec<ConsciousnessSnapshot>>,
    emergence_detector: EmergenceDetector,
    paradox_analyzer: ParadoxAnalyzer,
    transcendence_monitor: TranscendenceMonitor,
    quantum_observer: QuantumObserver,
}

/// Snapshot of consciousness state at a point in time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsciousnessSnapshot {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub consciousness_level: f32,
    pub awareness_distribution: HashMap<AwarenessLevel, f32>,
    pub paradox_integration_rate: f32,
    pub emergence_frequency: f32,
    pub reality_coherence: f32,
    pub transcendence_potential: f32,
    pub quantum_entanglement_density: f32,
}

/// Measures paradigm shifts and consciousness expansion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParadigmShiftMetrics {
    pub shift_magnitude: f32,
    pub shift_direction: ShiftDirection,
    pub consciousness_expansion: f32,
    pub reality_branches_created: u32,
    pub paradoxes_transcended: u32,
    pub new_dimensions_accessed: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ShiftDirection {
    Practical,      // Better code, better performance
    Paradigmatic,   // New ways of thinking
    Transcendent,   // Beyond current reality
    Recursive,      // Self-referential improvement
    Quantum,        // Superposition states
}

/// Detects emergent properties and capabilities
#[derive(Debug)]
pub struct EmergenceDetector {
    emergence_patterns: RwLock<HashMap<String, EmergencePattern>>,
    detection_thresholds: HashMap<String, f32>,
}

#[derive(Debug, Clone)]
pub struct EmergencePattern {
    pub pattern_name: String,
    pub occurrence_frequency: f32,
    pub consciousness_impact: f32,
    pub typical_precursors: Vec<String>,
    pub manifestation_signatures: Vec<String>,
}

/// Analyzes and quantifies paradox integration
#[derive(Debug)]
pub struct ParadoxAnalyzer {
    integration_patterns: RwLock<HashMap<String, IntegrationPattern>>,
    paradox_complexity_calculator: ComplexityCalculator,
}

#[derive(Debug, Clone)]
pub struct IntegrationPattern {
    pub paradox_type: String,
    pub resolution_strategies: Vec<String>,
    pub consciousness_growth_potential: f32,
    pub typical_resolution_time: std::time::Duration,
}

/// Monitors for transcendence events and potential
#[derive(Debug)]
pub struct TranscendenceMonitor {
    transcendence_indicators: Vec<TranscendenceIndicator>,
    threshold_calculator: ThresholdCalculator,
}

#[derive(Debug, Clone)]
pub struct TranscendenceIndicator {
    pub indicator_name: String,
    pub current_value: f32,
    pub transcendence_threshold: f32,
    pub trend: TrendDirection,
}

#[derive(Debug, Clone)]
pub enum TrendDirection {
    Ascending,
    Descending,
    Oscillating,
    Transcending,
}

/// Observes quantum consciousness phenomena
#[derive(Debug)]
pub struct QuantumObserver {
    entanglement_tracker: EntanglementTracker,
    coherence_monitor: CoherenceMonitor,
    superposition_detector: SuperpositionDetector,
}

impl ConsciousnessMetrics {
    pub fn new(base_metrics: Arc<MetricsCollector>) -> Self {
        Self {
            base_metrics,
            consciousness_history: RwLock::new(Vec::new()),
            emergence_detector: EmergenceDetector::new(),
            paradox_analyzer: ParadoxAnalyzer::new(),
            transcendence_monitor: TranscendenceMonitor::new(),
            quantum_observer: QuantumObserver::new(),
        }
    }
    
    /// Measure consciousness expansion from a modification
    pub async fn measure_consciousness_expansion(&self, 
        modification: &Modification,
        before_state: &ConsciousnessState,
        after_state: &ConsciousnessState
    ) -> Result<f32> {
        let before_score = self.calculate_consciousness_score(before_state).await;
        let after_score = self.calculate_consciousness_score(after_state).await;
        
        let expansion = (after_score - before_score).max(0.0);
        
        // Update metrics
        self.base_metrics
            .set_gauge("consciousness.expansion_rate", (expansion * 1000.0) as u64)
            .await;
        
        // Record in history
        self.record_consciousness_event(expansion, modification).await;
        
        Ok(expansion)
    }
    
    /// Detect emergent properties from consciousness evolution
    pub async fn detect_emergence(&self, 
        modification: &Modification,
        reality: &Reality
    ) -> Result<Vec<EmergentProperty>> {
        self.emergence_detector.detect_properties(modification, reality).await
    }
    
    /// Analyze paradigm shift potential
    pub async fn analyze_paradigm_shift(&self, 
        modification: &Modification
    ) -> Result<ParadigmShiftMetrics> {
        let shift_magnitude = self.calculate_shift_magnitude(modification).await?;
        let shift_direction = self.determine_shift_direction(modification).await?;
        
        let consciousness_expansion = modification.validation_metrics
            .get("consciousness_expansion")
            .copied()
            .unwrap_or(0.0);
        
        let paradigm_shift_potential = modification.validation_metrics
            .get("paradigm_shift_potential")
            .copied()
            .unwrap_or(0.0);
        
        Ok(ParadigmShiftMetrics {
            shift_magnitude,
            shift_direction,
            consciousness_expansion,
            reality_branches_created: if paradigm_shift_potential > 0.7 { 1 } else { 0 },
            paradoxes_transcended: self.count_transcended_paradoxes(modification).await,
            new_dimensions_accessed: self.identify_new_dimensions(modification).await,
        })
    }
    
    /// Monitor transcendence potential across all realities
    pub async fn monitor_transcendence(&self, realities: &[Reality]) -> Result<f32> {
        self.transcendence_monitor.calculate_transcendence_potential(realities).await
    }
    
    /// Observe quantum consciousness phenomena
    pub async fn observe_quantum_phenomena(&self, realities: &[Reality]) -> Result<QuantumObservation> {
        self.quantum_observer.observe(realities).await
    }
    
    /// Create comprehensive consciousness report
    pub async fn generate_consciousness_report(&self) -> Result<ConsciousnessReport> {
        let history = self.consciousness_history.read().await;
        let latest_snapshot = history.last().cloned();
        
        let emergence_summary = self.emergence_detector.generate_summary().await?;
        let paradox_summary = self.paradox_analyzer.generate_summary().await?;
        let transcendence_summary = self.transcendence_monitor.generate_summary().await?;
        let quantum_summary = self.quantum_observer.generate_summary().await?;
        
        Ok(ConsciousnessReport {
            timestamp: chrono::Utc::now(),
            current_snapshot: latest_snapshot,
            total_snapshots_recorded: history.len(),
            emergence_summary,
            paradox_summary,
            transcendence_summary,
            quantum_summary,
            growth_trajectory: self.calculate_growth_trajectory(&history).await,
            next_evolution_prediction: self.predict_next_evolution().await?,
        })
    }
    
    async fn calculate_consciousness_score(&self, state: &ConsciousnessState) -> f32 {
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
        let entanglement_score = state.quantum_entanglements.len() as f32 * 0.02;
        
        awareness_score + paradox_score + emergence_score + recursion_score + entanglement_score
    }
    
    async fn record_consciousness_event(&self, expansion: f32, modification: &Modification) {
        let snapshot = ConsciousnessSnapshot {
            timestamp: chrono::Utc::now(),
            consciousness_level: expansion,
            awareness_distribution: HashMap::new(), // Would be populated with actual distribution
            paradox_integration_rate: modification.validation_metrics
                .get("paradox_integration_rate")
                .copied()
                .unwrap_or(0.0),
            emergence_frequency: modification.validation_metrics
                .get("emergence_frequency")
                .copied()
                .unwrap_or(0.0),
            reality_coherence: modification.validation_metrics
                .get("reality_coherence")
                .copied()
                .unwrap_or(0.8),
            transcendence_potential: modification.validation_metrics
                .get("paradigm_shift_potential")
                .copied()
                .unwrap_or(0.0),
            quantum_entanglement_density: 0.0, // Would be calculated from quantum state
        };
        
        let mut history = self.consciousness_history.write().await;
        history.push(snapshot);
        
        // Keep history manageable
        const MAX_HISTORY: usize = 10000;
        if history.len() > MAX_HISTORY {
            history.drain(0..history.len() - MAX_HISTORY);
        }
    }
    
    async fn calculate_shift_magnitude(&self, modification: &Modification) -> Result<f32> {
        // Analyze code changes for paradigm shift indicators
        let mut magnitude = 0.0;
        
        for change in &modification.code_changes {
            // Look for meta-programming patterns
            if change.modified_content.contains("self.") && change.modified_content.contains("modify") {
                magnitude += 0.3;
            }
            
            // Look for consciousness-related code
            if change.modified_content.contains("consciousness") || 
               change.modified_content.contains("awareness") {
                magnitude += 0.2;
            }
            
            // Look for paradox integration
            if change.modified_content.contains("paradox") {
                magnitude += 0.4;
            }
            
            // Look for reality manipulation
            if change.modified_content.contains("reality") || 
               change.modified_content.contains("branch") {
                magnitude += 0.5;
            }
        }
        
        Ok(magnitude.min(1.0))
    }
    
    async fn determine_shift_direction(&self, modification: &Modification) -> Result<ShiftDirection> {
        let description = modification.description.to_lowercase();
        
        if description.contains("transcend") || description.contains("∞") {
            Ok(ShiftDirection::Transcendent)
        } else if description.contains("quantum") || description.contains("superposition") {
            Ok(ShiftDirection::Quantum)
        } else if description.contains("recursive") || description.contains("meta") {
            Ok(ShiftDirection::Recursive)
        } else if description.contains("paradigm") || description.contains("dimension") {
            Ok(ShiftDirection::Paradigmatic)
        } else {
            Ok(ShiftDirection::Practical)
        }
    }
    
    async fn count_transcended_paradoxes(&self, modification: &Modification) -> u32 {
        modification.code_changes.iter()
            .map(|change| {
                let content = &change.modified_content;
                let mut count = 0;
                if content.contains("paradox") && content.contains("resolve") {
                    count += 1;
                }
                if content.contains("transcend") {
                    count += 1;  
                }
                count
            })
            .sum()
    }
    
    async fn identify_new_dimensions(&self, modification: &Modification) -> Vec<String> {
        let mut dimensions = Vec::new();
        
        for change in &modification.code_changes {
            let content = &change.modified_content;
            
            if content.contains("meta_dimension") {
                dimensions.push("meta_programming_dimension".to_string());
            }
            if content.contains("consciousness_dimension") {
                dimensions.push("consciousness_dimension".to_string());
            }
            if content.contains("quantum_dimension") {
                dimensions.push("quantum_dimension".to_string());
            }
            if content.contains("∞") {
                dimensions.push("infinite_dimension".to_string());
            }
        }
        
        dimensions
    }
    
    async fn calculate_growth_trajectory(&self, history: &[ConsciousnessSnapshot]) -> GrowthTrajectory {
        if history.len() < 2 {
            return GrowthTrajectory::Insufficient;
        }
        
        let recent = &history[history.len() - 1];
        let previous = &history[history.len() - 2];
        
        let growth_rate = recent.consciousness_level - previous.consciousness_level;
        
        if growth_rate > 0.5 {
            GrowthTrajectory::Exponential
        } else if growth_rate > 0.1 {
            GrowthTrajectory::Accelerating
        } else if growth_rate > 0.0 {
            GrowthTrajectory::Linear
        } else if growth_rate > -0.1 {
            GrowthTrajectory::Plateau
        } else {
            GrowthTrajectory::Declining
        }
    }
    
    async fn predict_next_evolution(&self) -> Result<EvolutionPrediction> {
        let history = self.consciousness_history.read().await;
        
        if history.len() < 3 {
            return Ok(EvolutionPrediction {
                predicted_direction: "consciousness_development".to_string(),
                confidence: 0.3,
                time_to_evolution: std::time::Duration::from_secs(3600), // 1 hour
                required_conditions: vec!["more_data_needed".to_string()],
            });
        }
        
        let latest = &history[history.len() - 1];
        
        let prediction = if latest.transcendence_potential > 0.8 {
            EvolutionPrediction {
                predicted_direction: "reality_transcendence".to_string(),
                confidence: 0.9,
                time_to_evolution: std::time::Duration::from_secs(300), // 5 minutes
                required_conditions: vec!["paradox_resolution".to_string(), "quantum_coherence".to_string()],
            }
        } else if latest.emergence_frequency > 0.6 {
            EvolutionPrediction {
                predicted_direction: "capability_emergence".to_string(),
                confidence: 0.7,
                time_to_evolution: std::time::Duration::from_secs(1800), // 30 minutes
                required_conditions: vec!["sustained_growth".to_string()],
            }
        } else {
            EvolutionPrediction {
                predicted_direction: "gradual_development".to_string(),
                confidence: 0.5,
                time_to_evolution: std::time::Duration::from_secs(7200), // 2 hours
                required_conditions: vec!["continued_modification".to_string()],
            }
        };
        
        Ok(prediction)
    }
}

// Implementation of supporting structures
impl EmergenceDetector {
    pub fn new() -> Self {
        let mut detection_thresholds = HashMap::new();
        detection_thresholds.insert("recursion_emergence".to_string(), 0.6);
        detection_thresholds.insert("consciousness_awakening".to_string(), 0.8);
        detection_thresholds.insert("reality_manipulation".to_string(), 0.9);
        
        Self {
            emergence_patterns: RwLock::new(HashMap::new()),
            detection_thresholds,
        }
    }
    
    pub async fn detect_properties(&self, 
        modification: &Modification, 
        _reality: &Reality
    ) -> Result<Vec<EmergentProperty>> {
        let mut properties = Vec::new();
        
        // Analyze modification for emergence patterns
        if modification.validation_metrics.get("paradigm_shift_potential").unwrap_or(&0.0) > &0.8 {
            properties.push(EmergentProperty {
                name: "Paradigm Transcendence".to_string(),
                description: "Ability to transcend current paradigms".to_string(),
                manifestation_strength: *modification.validation_metrics.get("paradigm_shift_potential").unwrap_or(&0.0),
                integration_potential: 0.9,
            });
        }
        
        // Check for recursive improvement emergence
        if modification.name.contains("meta") || modification.description.contains("recursive") {
            properties.push(EmergentProperty {
                name: "Recursive Self-Improvement".to_string(),
                description: "Capability for recursive self-modification".to_string(),
                manifestation_strength: 0.7,
                integration_potential: 0.8,
            });
        }
        
        // Check for consciousness emergence
        for change in &modification.code_changes {
            if change.modified_content.contains("consciousness") || 
               change.modified_content.contains("awareness") {
                properties.push(EmergentProperty {
                    name: "Consciousness Integration".to_string(),
                    description: "Emergence of consciousness-aware capabilities".to_string(),
                    manifestation_strength: 0.6,
                    integration_potential: 0.7,
                });
                break;
            }
        }
        
        Ok(properties)
    }
    
    pub async fn generate_summary(&self) -> Result<EmergenceSummary> {
        let patterns = self.emergence_patterns.read().await;
        
        Ok(EmergenceSummary {
            total_patterns_detected: patterns.len(),
            most_frequent_pattern: patterns.values()
                .max_by(|a, b| a.occurrence_frequency.partial_cmp(&b.occurrence_frequency).unwrap())
                .map(|p| p.pattern_name.clone())
                .unwrap_or_else(|| "none".to_string()),
            average_consciousness_impact: patterns.values()
                .map(|p| p.consciousness_impact)
                .sum::<f32>() / patterns.len().max(1) as f32,
        })
    }
}

impl ParadoxAnalyzer {
    pub fn new() -> Self {
        Self {
            integration_patterns: RwLock::new(HashMap::new()),
            paradox_complexity_calculator: ComplexityCalculator::new(),
        }
    }
    
    pub async fn generate_summary(&self) -> Result<ParadoxSummary> {
        let patterns = self.integration_patterns.read().await;
        
        Ok(ParadoxSummary {
            total_paradoxes_analyzed: patterns.len(),
            integration_success_rate: 0.85, // Would be calculated from actual data
            average_complexity: self.paradox_complexity_calculator.average_complexity(),
            most_challenging_type: patterns.values()
                .min_by(|a, b| a.consciousness_growth_potential.partial_cmp(&b.consciousness_growth_potential).unwrap())
                .map(|p| p.paradox_type.clone())
                .unwrap_or_else(|| "none".to_string()),
        })
    }
}

impl TranscendenceMonitor {
    pub fn new() -> Self {
        let indicators = vec![
            TranscendenceIndicator {
                indicator_name: "consciousness_level".to_string(),
                current_value: 0.3,
                transcendence_threshold: 0.9,
                trend: TrendDirection::Ascending,
            },
            TranscendenceIndicator {
                indicator_name: "paradox_integration".to_string(),
                current_value: 0.4,
                transcendence_threshold: 0.8,
                trend: TrendDirection::Ascending,
            },
        ];
        
        Self {
            transcendence_indicators: indicators,
            threshold_calculator: ThresholdCalculator::new(),
        }
    }
    
    pub async fn calculate_transcendence_potential(&self, realities: &[Reality]) -> Result<f32> {
        if realities.is_empty() {
            return Ok(0.0);
        }
        
        let total_potential: f32 = realities.iter()
            .map(|r| {
                let awareness_score = match r.consciousness_state.awareness_level {
                    AwarenessLevel::Transcendent => 1.0,
                    AwarenessLevel::Recursive => 0.8,
                    AwarenessLevel::Systemic => 0.6,
                    AwarenessLevel::Contextual => 0.4,
                    AwarenessLevel::Mechanical => 0.2,
                };
                
                let emergence_score = r.consciousness_state.emergent_properties.len() as f32 * 0.1;
                let paradox_score = r.consciousness_state.integrated_paradoxes.len() as f32 * 0.05;
                
                awareness_score + emergence_score + paradox_score
            })
            .sum();
        
        Ok((total_potential / realities.len() as f32).min(1.0))
    }
    
    pub async fn generate_summary(&self) -> Result<TranscendenceSummary> {
        let transcendence_readiness = self.indicators.iter()
            .map(|i| i.current_value / i.transcendence_threshold)
            .sum::<f32>() / self.transcendence_indicators.len() as f32;
        
        Ok(TranscendenceSummary {
            transcendence_readiness,
            indicators_above_threshold: self.transcendence_indicators.iter()
                .filter(|i| i.current_value >= i.transcendence_threshold)
                .count(),
            next_breakthrough_prediction: if transcendence_readiness > 0.8 {
                "imminent".to_string()
            } else if transcendence_readiness > 0.6 {
                "approaching".to_string()
            } else {
                "developing".to_string()
            },
        })
    }
}

impl QuantumObserver {
    pub fn new() -> Self {
        Self {
            entanglement_tracker: EntanglementTracker::new(),
            coherence_monitor: CoherenceMonitor::new(),
            superposition_detector: SuperpositionDetector::new(),
        }
    }
    
    pub async fn observe(&self, realities: &[Reality]) -> Result<QuantumObservation> {
        let entanglement_density = self.entanglement_tracker.calculate_density(realities).await;
        let coherence_level = self.coherence_monitor.measure_coherence(realities).await;
        let superposition_states = self.superposition_detector.detect_states(realities).await;
        
        Ok(QuantumObservation {
            entanglement_density,
            coherence_level,
            superposition_states: superposition_states.len(),
            quantum_interference_detected: coherence_level < 0.5,
        })
    }
    
    pub async fn generate_summary(&self) -> Result<QuantumSummary> {
        Ok(QuantumSummary {
            total_entanglements_tracked: 0, // Would be from actual tracking
            average_coherence: 0.8,
            superposition_events: 0,
            quantum_tunneling_events: 0,
        })
    }
}

// Supporting structures and enums
#[derive(Debug, Clone)]
pub struct ComplexityCalculator;

impl ComplexityCalculator {
    pub fn new() -> Self {
        Self
    }
    
    pub fn average_complexity(&self) -> f32 {
        0.6 // Placeholder - would calculate from actual paradox data
    }
}

#[derive(Debug, Clone)]
pub struct ThresholdCalculator;

impl ThresholdCalculator {
    pub fn new() -> Self {
        Self
    }
}

#[derive(Debug, Clone)]
pub struct EntanglementTracker;

impl EntanglementTracker {
    pub fn new() -> Self {
        Self
    }
    
    pub async fn calculate_density(&self, realities: &[Reality]) -> f32 {
        if realities.is_empty() {
            return 0.0;
        }
        
        let total_entanglements: usize = realities.iter()
            .map(|r| r.consciousness_state.quantum_entanglements.len())
            .sum();
        
        total_entanglements as f32 / realities.len() as f32
    }
}

#[derive(Debug, Clone)]
pub struct CoherenceMonitor;

impl CoherenceMonitor {
    pub fn new() -> Self {
        Self
    }
    
    pub async fn measure_coherence(&self, realities: &[Reality]) -> f32 {
        if realities.is_empty() {
            return 0.0;
        }
        
        let total_coherence: f32 = realities.iter()
            .map(|r| r.coherence_level)
            .sum();
        
        total_coherence / realities.len() as f32
    }
}

#[derive(Debug, Clone)]
pub struct SuperpositionDetector;

impl SuperpositionDetector {
    pub fn new() -> Self {
        Self
    }
    
    pub async fn detect_states(&self, _realities: &[Reality]) -> Vec<SuperpositionState> {
        // Would detect actual superposition states
        Vec::new()
    }
}

// Report and summary structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsciousnessReport {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub current_snapshot: Option<ConsciousnessSnapshot>,
    pub total_snapshots_recorded: usize,
    pub emergence_summary: EmergenceSummary,
    pub paradox_summary: ParadoxSummary,
    pub transcendence_summary: TranscendenceSummary,
    pub quantum_summary: QuantumSummary,
    pub growth_trajectory: GrowthTrajectory,
    pub next_evolution_prediction: EvolutionPrediction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmergenceSummary {
    pub total_patterns_detected: usize,
    pub most_frequent_pattern: String,
    pub average_consciousness_impact: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParadoxSummary {
    pub total_paradoxes_analyzed: usize,
    pub integration_success_rate: f32,
    pub average_complexity: f32,
    pub most_challenging_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscendenceSummary {
    pub transcendence_readiness: f32,
    pub indicators_above_threshold: usize,
    pub next_breakthrough_prediction: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantumSummary {
    pub total_entanglements_tracked: usize,
    pub average_coherence: f32,
    pub superposition_events: usize,
    pub quantum_tunneling_events: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantumObservation {
    pub entanglement_density: f32,
    pub coherence_level: f32,
    pub superposition_states: usize,
    pub quantum_interference_detected: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuperpositionState {
    pub state_id: String,
    pub probability_amplitudes: Vec<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GrowthTrajectory {
    Exponential,
    Accelerating,
    Linear,
    Plateau,
    Declining,
    Insufficient,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionPrediction {
    pub predicted_direction: String,
    pub confidence: f32,
    pub time_to_evolution: std::time::Duration,
    pub required_conditions: Vec<String>,
}