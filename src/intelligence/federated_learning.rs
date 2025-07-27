use crate::core::vector::Vector;
use std::collections::HashMap;

pub struct Model {
    pub weights: Vec<f32>,
}

impl Model {
    pub fn new(dimensions: usize) -> Self {
        Self {
            weights: vec![0.0; dimensions],
        }
    }
}

pub struct Client {
    pub id: String,
    pub model: Model,
    pub data: Vec<Vector>,
}

impl Client {
    pub fn new(id: &str, dimensions: usize, data: Vec<Vector>) -> Self {
        Self {
            id: id.to_string(),
            model: Model::new(dimensions),
            data,
        }
    }
}

/// Federated learning coordinator placeholder
pub struct FederatedLearning {
    pub global_model: Model,
    pub clients: HashMap<String, Client>,
    pub mu: f32,
}

impl FederatedLearning {
    /// Create a new federated learning coordinator
    pub fn new(dimensions: usize, mu: f32) -> Self {
        Self {
            global_model: Model::new(dimensions),
            clients: HashMap::new(),
            mu,
        }
    }

    pub fn add_client(&mut self, client: Client) {
        self.clients.insert(client.id.clone(), client);
    }

    pub fn train(&mut self, rounds: usize) {
        for _ in 0..rounds {
            let mut updates = Vec::new();
            for client in self.clients.values_mut() {
                let update = self.train_client(client);
                updates.push(update);
            }
            self.aggregate(updates);
        }
    }

    fn train_client(&self, client: &mut Client) -> Model {
        // In a real implementation, this would train the client's model on its local data.
        // For now, we'll just return a copy of the client's model with the proximal term applied.
        let mut new_weights = client.model.weights.clone();
        for (i, weight) in new_weights.iter_mut().enumerate() {
            *weight -= self.mu * (client.model.weights[i] - self.global_model.weights[i]);
        }
        Model {
            weights: new_weights,
        }
    }

    fn aggregate(&mut self, updates: Vec<Model>) {
        // In a real implementation, this would aggregate the updates from the clients.
        // For now, we'll just average the weights.
        let mut new_weights = vec![0.0; self.global_model.weights.len()];
        for update in &updates {
            for (i, weight) in update.weights.iter().enumerate() {
                new_weights[i] += weight;
            }
        }
        for weight in &mut new_weights {
            *weight /= updates.len() as f32;
        }
        self.global_model.weights = new_weights;
    }
}

impl Clone for Model {
    fn clone(&self) -> Self {
        Self {
            weights: self.weights.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_train_client_updates_toward_global() {
        let dimensions = 3;
        let mu = 0.5;

        let mut fl = FederatedLearning::new(dimensions, mu);
        fl.global_model.weights = vec![2.0, 2.0, 2.0];

        let mut client = Client::new("c1", dimensions, Vec::new());
        client.model.weights = vec![0.0, 0.0, 0.0];

        let updated = fl.train_client(&mut client);

        assert_eq!(updated.weights, vec![1.0, 1.0, 1.0]);
        // Ensure client weights are unchanged
        assert_eq!(client.model.weights, vec![0.0, 0.0, 0.0]);
    }

    #[test]
    fn test_aggregate_averages_updates() {
        let dimensions = 2;
        let mu = 0.0;

        let mut fl = FederatedLearning::new(dimensions, mu);

        let updates = vec![
            Model { weights: vec![1.0, 3.0] },
            Model { weights: vec![3.0, 1.0] },
        ];

        fl.aggregate(updates);

        assert_eq!(fl.global_model.weights, vec![2.0, 2.0]);
    }
}
