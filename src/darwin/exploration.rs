use anyhow::{anyhow, Result};
use dashmap::DashMap;
use rand::prelude::*;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::core::metrics::MetricsCollector;
use crate::darwin::self_improvement::Modification;

/// Strategy for exploring potential system improvements
#[derive(Debug)]
pub struct ExplorationStrategy {
    /// Metrics collector
    metrics: Arc<MetricsCollector>,

    /// Archive of previously explored solutions
    archive: DashMap<String, ArchiveEntry>,

    /// Current exploration parameters
    parameters: RwLock<ExplorationParameters>,

    /// Novelty archive for quality-diversity
    novelty_archive: RwLock<Vec<NoveltyPoint>>,
}

#[derive(Debug, Clone)]
struct ArchiveEntry {
    /// The modification
    modification: Modification,

    /// Performance metrics
    metrics: HashMap<String, f32>,

    /// Feature descriptors for quality-diversity
    features: HashMap<String, f32>,

    /// When this entry was added
    added_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone)]
struct ExplorationParameters {
    /// Mutation rate
    mutation_rate: f32,

    /// Crossover rate
    crossover_rate: f32,

    /// Novelty threshold
    novelty_threshold: f32,

    /// Maximum archive size
    max_archive_size: usize,

    /// Number of parents for tournament selection
    tournament_size: usize,

    /// Probability of selecting a random direction
    exploration_rate: f32,
}

/// Point in novelty space
#[derive(Debug, Clone)]
struct NoveltyPoint {
    /// ID of the solution
    id: Uuid,

    /// Feature vector describing the solution
    features: HashMap<String, f32>,

    /// Performance score
    score: f32,

    /// When this point was added
    added_at: chrono::DateTime<chrono::Utc>,
}

impl ExplorationStrategy {
    pub fn new(metrics: Arc<MetricsCollector>) -> Self {
        Self {
            metrics,
            archive: DashMap::new(),
            parameters: RwLock::new(ExplorationParameters {
                mutation_rate: 0.1,
                crossover_rate: 0.3,
                novelty_threshold: 0.2,
                max_archive_size: 1000,
                tournament_size: 3,
                exploration_rate: 0.2,
            }),
            novelty_archive: RwLock::new(Vec::new()),
        }
    }

    /// Generate new modification proposals
    pub async fn generate_proposals(&self) -> Result<Vec<Modification>> {
        let mut proposals = Vec::new();

        // Snapshot the current archive
        let archive = self.archive.clone();

        // Get current archive length
        let archive_len = archive.len();
        let params = self.parameters.read().await;

        // If archive is empty, generate some initial proposals
        if archive_len == 0 {
            proposals.extend(self.generate_initial_proposals().await?);
        } else {
            // Generate proposals through various strategies

            // 1. Mutation of existing solutions

            let mutation_count = (archive_len as f32 * params.mutation_rate).ceil() as usize;
            proposals.extend(self.generate_mutations(&archive, mutation_count).await?);

            // 2. Crossover between existing solutions
            let crossover_count = (archive_len as f32 * params.crossover_rate).ceil() as usize;
            proposals.extend(self.generate_crossovers(&archive, crossover_count).await?);

            // 3. Novelty search for unexplored areas
            proposals.extend(self.generate_novelty_search(&archive).await?);
        }

        // Update metrics
        self.metrics
            .increment_counter(
                "darwin.exploration.proposals_generated",
                proposals.len() as u64,
            )
            .await;

        info!("Generated {} new modification proposals", proposals.len());

        Ok(proposals)
    }

    /// Generate initial proposals when archive is empty
    async fn generate_initial_proposals(&self) -> Result<Vec<Modification>> {
        // In a real implementation, this would generate initial proposals
        // based on system analysis or predefined templates

        // For now, we'll create a simple example proposal
        let mut proposals = Vec::new();

        let proposal = Modification {
            id: Uuid::new_v4(),
            name: "Initial optimization".to_string(),
            description: "Optimize vector search algorithm".to_string(),
            code_changes: Vec::new(), // Would contain actual code changes
            validation_metrics: HashMap::new(),
            created_at: chrono::Utc::now(),
            status: crate::darwin::self_improvement::ModificationStatus::Proposed,
        };

        proposals.push(proposal);

        Ok(proposals)
    }

    /// Generate mutations of existing solutions
    async fn generate_mutations(
        &self,
        archive: &DashMap<String, ArchiveEntry>,
        count: usize,
    ) -> Result<Vec<Modification>> {
        let mut proposals = Vec::new();
        let mut rng = rand::thread_rng();

        // Select random entries to mutate

        let entries: Vec<ArchiveEntry> = archive.iter().map(|e| e.value().clone()).collect();

        for _ in 0..count {
            if let Some(entry) = entries.choose(&mut rng) {
                let mut proposal = entry.modification.clone();

                // Update fields for the new proposal
                proposal.id = Uuid::new_v4();
                proposal.name = format!("Mutation of {}", entry.modification.name);
                proposal.description =
                    format!("Mutated version of {}", entry.modification.description);
                proposal.created_at = chrono::Utc::now();
                proposal.status = crate::darwin::self_improvement::ModificationStatus::Proposed;

                // In a real implementation, we would actually mutate the code changes

                proposals.push(proposal);
            }
        }

        Ok(proposals)
    }

    /// Generate crossovers between existing solutions
    async fn generate_crossovers(
        &self,
        archive: &DashMap<String, ArchiveEntry>,
        count: usize,
    ) -> Result<Vec<Modification>> {
        let mut proposals = Vec::new();
        let mut rng = rand::thread_rng();

        // Select random pairs of entries to crossover
        let entries: Vec<ArchiveEntry> = archive.iter().map(|e| e.value().clone()).collect();

        for _ in 0..count {
            if entries.len() < 2 {
                break;
            }

            let parent1 = entries
                .choose(&mut rng)
                .ok_or_else(|| anyhow!("archive is empty"))?;
            let parent2 = entries
                .choose(&mut rng)
                .ok_or_else(|| anyhow!("archive is empty"))?;

            let proposal = Modification {
                id: Uuid::new_v4(),
                name: format!(
                    "Crossover of {} and {}",
                    parent1.modification.name, parent2.modification.name
                ),
                description: format!("Combined features from multiple parent modifications"),
                code_changes: Vec::new(), // Would contain actual code changes from crossover
                validation_metrics: HashMap::new(),
                created_at: chrono::Utc::now(),
                status: crate::darwin::self_improvement::ModificationStatus::Proposed,
            };

            proposals.push(proposal);
        }

        Ok(proposals)
    }

    /// Generate proposals using novelty search
    async fn generate_novelty_search(
        &self,
        archive: &DashMap<String, ArchiveEntry>,
    ) -> Result<Vec<Modification>> {
        // In a real implementation, this would use novelty search to explore
        // underrepresented areas of the solution space

        // Get the novelty archive
        let novelty_archive = self.novelty_archive.read().await;

        // If the novelty archive is empty, just return a simple proposal
        if novelty_archive.is_empty() {
            let proposal = Modification {
                id: Uuid::new_v4(),
                name: "Novelty search proposal".to_string(),
                description: "Exploring new optimization strategies".to_string(),
                code_changes: Vec::new(), // Would contain actual code changes
                validation_metrics: HashMap::new(),
                created_at: chrono::Utc::now(),
                status: crate::darwin::self_improvement::ModificationStatus::Proposed,
            };

            return Ok(vec![proposal]);
        }

        // Find sparse areas in the feature space
        // For now, this is a simplified implementation
        let feature_keys: HashSet<String> = novelty_archive
            .iter()
            .flat_map(|point| point.features.keys().cloned())
            .collect();

        let mut feature_means: HashMap<String, f32> = HashMap::new();
        let mut feature_counts: HashMap<String, usize> = HashMap::new();

        // Calculate mean for each feature
        for point in novelty_archive.iter() {
            for (key, value) in &point.features {
                let count = feature_counts.entry(key.clone()).or_insert(0);
                *count += 1;

                let sum = feature_means.entry(key.clone()).or_insert(0.0);
                *sum += value;
            }
        }

        // Finalize means
        for (key, sum) in &mut feature_means {
            if let Some(count) = feature_counts.get(key) {
                if *count > 0 {
                    *sum /= *count as f32;
                }
            }
        }

        // Generate a proposal that's different from the mean
        let mut target_features = HashMap::new();
        for key in feature_keys {
            if let Some(mean) = feature_means.get(&key) {
                // Aim for a value that's different from the mean
                let mut rng = rand::thread_rng();
                let direction = if rng.gen::<f32>() > 0.5 { 1.0 } else { -1.0 };
                let magnitude = rng.gen::<f32>() * 0.5 + 0.5; // 0.5 to 1.0

                target_features.insert(key, mean + direction * magnitude);
            }
        }

        // Create a proposal aiming for these target features
        let proposal = Modification {
            id: Uuid::new_v4(),
            name: "Novelty search proposal".to_string(),
            description: format!("Exploring new optimization strategies with targeted features"),
            code_changes: Vec::new(), // Would contain actual code changes
            validation_metrics: HashMap::new(),
            created_at: chrono::Utc::now(),
            status: crate::darwin::self_improvement::ModificationStatus::Proposed,
        };

        Ok(vec![proposal])
    }

    /// Tournament selection for choosing parents
    async fn tournament_selection(
        &self,
        archive: &DashMap<String, ArchiveEntry>,
    ) -> Option<ArchiveEntry> {
        let params = self.parameters.read().await;
        let mut rng = rand::thread_rng();

        // If archive is too small, just return a random entry
        if archive.len() <= 1 {
            return archive.iter().next().map(|e| e.value().clone());
        }

        let entries: Vec<ArchiveEntry> = archive.iter().map(|e| e.value().clone()).collect();

        // Select tournament_size random entries
        let tournament_size = std::cmp::min(params.tournament_size, entries.len());
        let mut tournament = Vec::with_capacity(tournament_size);

        for _ in 0..tournament_size {
            if let Some(entry) = entries.choose(&mut rng) {
                tournament.push(entry.clone());
            }
        }

        // Find the best entry in the tournament
        tournament.into_iter().max_by(|a, b| {
            // Compare based on sum of metrics (higher is better)
            let a_score: f32 = a.metrics.values().sum();
            let b_score: f32 = b.metrics.values().sum();
            a_score
                .partial_cmp(&b_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    /// Add a validated modification to the archive
    pub async fn add_to_archive(
        &self,
        modification: Modification,
        metrics: HashMap<String, f32>,
    ) -> Result<()> {
        let params = self.parameters.read().await;

        // Generate feature descriptors for quality-diversity
        let features = self.extract_features(&modification, &metrics);
        // Keep a copy for the archive entry before moving features elsewhere
        let archive_features = features.clone();

        // Precompute fields used before moving values
        let mod_id = modification.id;
        let score = metrics.values().sum();

        // Create archive entry
        let entry = ArchiveEntry {
            modification,
            metrics,
            features: archive_features,
            added_at: chrono::Utc::now(),
        };

        // Add to archive
        let key = entry.modification.id.to_string();

        self.archive.insert(key, entry);

        // Add to novelty archive
        let novelty_point = NoveltyPoint {
            id: mod_id,
            features,
            score,
            added_at: chrono::Utc::now(),
        };

        {
            let mut novelty_archive = self.novelty_archive.write().await;
            novelty_archive.push(novelty_point);

            // Keep novelty archive sorted by score (descending)
            novelty_archive.sort_by(|a, b| {
                b.score
                    .partial_cmp(&a.score)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

            // Trim novelty archive if needed
            if novelty_archive.len() > params.max_archive_size {
                novelty_archive.truncate(params.max_archive_size);
            }
        }

        // Trim archive if needed
        if self.archive.len() > params.max_archive_size {
            self.trim_archive().await?;
        }

        // Update metrics
        self.metrics
            .set_gauge("darwin.exploration.archive_size", self.archive.len() as u64)
            .await;

        Ok(())
    }

    /// Extract feature descriptors for quality-diversity
    fn extract_features(
        &self,
        modification: &Modification,
        metrics: &HashMap<String, f32>,
    ) -> HashMap<String, f32> {
        let mut features = HashMap::new();

        // In a real implementation, this would extract meaningful features
        // that describe the solution in a way that promotes diversity

        // For now, we'll use some example features
        features.insert(
            "code_size".to_string(),
            modification.code_changes.len() as f32,
        );

        if let Some(latency) = metrics.get("performance.vector_search_latency_ms") {
            features.insert("latency".to_string(), *latency);
        }

        if let Some(throughput) = metrics.get("performance.throughput_qps") {
            features.insert("throughput".to_string(), *throughput);
        }

        features
    }

    /// Calculate novelty score for a solution
    async fn calculate_novelty_score(&self, features: &HashMap<String, f32>) -> f32 {
        let novelty_archive = self.novelty_archive.read().await;

        if novelty_archive.is_empty() {
            return 1.0; // Maximum novelty if archive is empty
        }

        // Calculate average distance to k-nearest neighbors
        let k = std::cmp::min(15, novelty_archive.len());

        let mut distances = Vec::with_capacity(novelty_archive.len());

        for point in novelty_archive.iter() {
            let distance = self.feature_distance(features, &point.features);
            distances.push(distance);
        }

        // Sort distances and take the k smallest
        distances.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let avg_distance = distances.iter().take(k).sum::<f32>() / k as f32;

        avg_distance
    }

    /// Calculate distance between feature vectors
    fn feature_distance(&self, a: &HashMap<String, f32>, b: &HashMap<String, f32>) -> f32 {
        let mut sum_squared_diff = 0.0;
        let mut feature_count = 0;

        // Get all unique keys
        let all_keys: HashSet<_> = a.keys().chain(b.keys()).collect();

        for key in all_keys {
            let a_val = a.get(key).copied().unwrap_or(0.0);
            let b_val = b.get(key).copied().unwrap_or(0.0);

            let diff = a_val - b_val;
            sum_squared_diff += diff * diff;
            feature_count += 1;
        }

        if feature_count == 0 {
            return 0.0;
        }

        (sum_squared_diff / feature_count as f32).sqrt()
    }

    /// Trim the archive to maintain diversity
    async fn trim_archive(&self) -> Result<()> {
        // In a real implementation, this would use quality-diversity
        // algorithms to maintain a diverse set of high-quality solutions

        // For now, we'll just keep the newest entries
        let mut entries: Vec<(String, ArchiveEntry)> = self
            .archive
            .iter()
            .map(|e| (e.key().clone(), e.value().clone()))
            .collect();
        entries.sort_by(|a, b| b.1.added_at.cmp(&a.1.added_at));

        let params = self.parameters.read().await;
        entries.truncate(params.max_archive_size);

        self.archive.clear();

        for (key, entry) in entries {
            self.archive.insert(key, entry);
        }

        Ok(())
    }

    /// Update exploration parameters
    pub async fn update_parameters(
        &self,
        mutation_rate: Option<f32>,
        crossover_rate: Option<f32>,
        novelty_threshold: Option<f32>,
        exploration_rate: Option<f32>,
    ) -> Result<()> {
        let mut params = self.parameters.write().await;

        if let Some(rate) = mutation_rate {
            params.mutation_rate = rate.max(0.0).min(1.0);
        }

        if let Some(rate) = crossover_rate {
            params.crossover_rate = rate.max(0.0).min(1.0);
        }

        if let Some(threshold) = novelty_threshold {
            params.novelty_threshold = threshold.max(0.0);
        }

        if let Some(rate) = exploration_rate {
            params.exploration_rate = rate.max(0.0).min(1.0);
        }

        info!("Updated exploration parameters: mutation_rate={:.2}, crossover_rate={:.2}, novelty_threshold={:.2}, exploration_rate={:.2}",
              params.mutation_rate, params.crossover_rate, params.novelty_threshold, params.exploration_rate);

        Ok(())
    }

    /// Get the current exploration parameters
    pub async fn get_parameters(&self) -> ExplorationParameters {
        self.parameters.read().await.clone()
    }

    /// Get statistics about the exploration archive
    pub async fn get_archive_stats(&self) -> ArchiveStats {
        let total_entries = self.archive.len();
        let novelty_archive = self.novelty_archive.read().await;

        ArchiveStats {
            total_entries,
            novelty_entries: novelty_archive.len(),
            feature_coverage: self.calculate_feature_coverage().await,
            top_scores: self.get_top_scores(5).await,
        }
    }

    /// Calculate feature coverage (how much of the feature space is explored)
    async fn calculate_feature_coverage(&self) -> f32 {
        // This is a placeholder - a real implementation would calculate
        // a more sophisticated coverage metric
        let novelty_archive = self.novelty_archive.read().await;

        if novelty_archive.is_empty() {
            return 0.0;
        }

        // Count unique features
        let feature_keys: HashSet<_> = novelty_archive
            .iter()
            .flat_map(|point| point.features.keys().cloned())
            .collect();

        // Simple coverage metric based on number of features and points
        let feature_count = feature_keys.len();
        let point_count = novelty_archive.len();

        if feature_count == 0 {
            return 0.0;
        }

        // Calculate dispersion of points for each feature
        let mut total_dispersion = 0.0;

        for key in feature_keys {
            let values: Vec<f32> = novelty_archive
                .iter()
                .filter_map(|point| point.features.get(&key).copied())
                .collect();

            if values.is_empty() {
                continue;
            }

            // Calculate min and max
            let min = values.iter().fold(f32::INFINITY, |a, &b| a.min(b));
            let max = values.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));

            if min.is_finite() && max.is_finite() && max > min {
                total_dispersion += (max - min);
            }
        }

        // Normalize by feature count
        total_dispersion / feature_count as f32
    }

    /// Get top scoring solutions
    async fn get_top_scores(&self, count: usize) -> Vec<(Uuid, f32)> {
        let novelty_archive = self.novelty_archive.read().await;

        novelty_archive
            .iter()
            .take(count)
            .map(|point| (point.id, point.score))
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct ArchiveStats {
    pub total_entries: usize,
    pub novelty_entries: usize,
    pub feature_coverage: f32,
    pub top_scores: Vec<(Uuid, f32)>,
}
