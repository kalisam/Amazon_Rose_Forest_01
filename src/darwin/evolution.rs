use crate::core::vector::Vector;
use rand::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Debug, Clone)]
struct Chromosome {
    genes: HashMap<String, f32>,
}

impl Chromosome {
    fn new(parameters: &HashMap<String, f32>) -> Self {
        Self {
            genes: parameters.clone(),
        }
    }

    fn crossover(&self, other: &Chromosome) -> Chromosome {
        let mut rng = rand::thread_rng();
        let mut child_genes = HashMap::new();

        for (key, value) in &self.genes {
            if rng.gen::<bool>() {
                child_genes.insert(key.clone(), *value);
            } else {
                child_genes.insert(key.clone(), *other.genes.get(key).unwrap_or(value));
            }
        }

        Chromosome { genes: child_genes }
    }

    fn mutate(&mut self, mutation_rate: f32, mutation_strength: f32) {
        let mut rng = rand::thread_rng();
        for (_, value) in self.genes.iter_mut() {
            if rng.gen::<f32>() < mutation_rate {
                *value += rng.gen_range(-mutation_strength..mutation_strength);
            }
        }
    }
}

#[derive(Debug)]
pub struct EvolutionEngine {
    models: RwLock<HashMap<Uuid, Model>>,
}

#[derive(Debug)]
struct Model {
    id: Uuid,
    name: String,
    version: u64,
    parameters: HashMap<String, f32>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl EvolutionEngine {
    pub fn new() -> Self {
        Self {
            models: RwLock::new(HashMap::new()),
        }
    }

    pub async fn create_model(&self, name: &str) -> Uuid {
        let id = Uuid::new_v4();
        let now = chrono::Utc::now();

        let model = Model {
            id,
            name: name.to_string(),
            version: 1,
            parameters: HashMap::new(),
            created_at: now,
            updated_at: now,
        };

        self.models.write().await.insert(id, model);
        id
    }

    pub async fn evolve_model(
        &self,
        model_id: Uuid,
        observations: Vec<Vector>,
    ) -> Result<(), String> {
        let mut models = self.models.write().await;
        let model = models
            .get_mut(&model_id)
            .ok_or(format!("Model with ID {} not found", model_id))?;

        // Update version
        model.version += 1;
        model.updated_at = chrono::Utc::now();

        // Simulate model evolution based on observations
        let mut population = Vec::new();
        for _ in 0..100 {
            population.push(Chromosome::new(&model.parameters));
        }

        for _ in 0..10 {
            let mut new_population = Vec::new();
            for _ in 0..100 {
                let parent1 = population.choose(&mut rand::thread_rng()).unwrap();
                let parent2 = population.choose(&mut rand::thread_rng()).unwrap();
                let mut child = parent1.crossover(parent2);
                child.mutate(0.1, 0.1);
                new_population.push(child);
            }
            population = new_population;
        }

        let best_chromosome = population
            .iter()
            .max_by(|a, b| {
                let a_fitness = self.fitness(a, &observations);
                let b_fitness = self.fitness(b, &observations);
                a_fitness.partial_cmp(&b_fitness).unwrap()
            })
            .unwrap();

        model.parameters = best_chromosome.genes.clone();

        Ok(())
    }

    fn fitness(&self, chromosome: &Chromosome, observations: &Vec<Vector>) -> f32 {
        let mut error = 0.0;
        for obs in observations {
            for (key, value) in &chromosome.genes {
                if let Some(obs_val) = obs.values.get(0) {
                    error += (value - obs_val).powi(2);
                }
            }
        }
        -error
    }

    pub async fn get_model_version(&self, model_id: Uuid) -> Result<u64, String> {
        let models = self.models.read().await;
        models
            .get(&model_id)
            .map(|model| model.version)
            .ok_or(format!("Model with ID {} not found", model_id))
    }
}
