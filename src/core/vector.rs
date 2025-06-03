use serde::{Deserialize, Serialize};
use std::ops::{Add, Div, Mul, Sub};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Vector {
    pub dimensions: usize,
    pub values: Vec<f32>,
}

impl Vector {
    pub fn new(values: Vec<f32>) -> Self {
        let dimensions = values.len();
        Self { dimensions, values }
    }
    
    pub fn zeros(dimensions: usize) -> Self {
        Self {
            dimensions,
            values: vec![0.0; dimensions],
        }
    }
    
    pub fn random(dimensions: usize) -> Self {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let values = (0..dimensions).map(|_| rng.gen::<f32>()).collect();
        Self { dimensions, values }
    }
    
    pub fn dot(&self, other: &Vector) -> f32 {
        assert_eq!(self.dimensions, other.dimensions, "Vectors must have the same dimensions");
        self.values.iter().zip(other.values.iter()).map(|(a, b)| a * b).sum()
    }
    
    pub fn magnitude(&self) -> f32 {
        self.dot(self).sqrt()
    }
    
    pub fn normalize(&self) -> Self {
        let mag = self.magnitude();
        if mag == 0.0 {
            return self.clone();
        }
        
        let values = self.values.iter().map(|v| v / mag).collect();
        Self {
            dimensions: self.dimensions,
            values,
        }
    }
    
    pub fn cosine_similarity(&self, other: &Vector) -> f32 {
        let dot_product = self.dot(other);
        let magnitude_product = self.magnitude() * other.magnitude();
        
        if magnitude_product == 0.0 {
            return 0.0;
        }
        
        dot_product / magnitude_product
    }
    
    pub fn euclidean_distance(&self, other: &Vector) -> f32 {
        assert_eq!(self.dimensions, other.dimensions, "Vectors must have the same dimensions");
        
        let sum_squared_diff: f32 = self.values
            .iter()
            .zip(other.values.iter())
            .map(|(a, b)| (a - b).powi(2))
            .sum();
            
        sum_squared_diff.sqrt()
    }
}

impl Add for Vector {
    type Output = Self;
    
    fn add(self, other: Self) -> Self::Output {
        assert_eq!(self.dimensions, other.dimensions, "Vectors must have the same dimensions");
        
        let values = self.values
            .iter()
            .zip(other.values.iter())
            .map(|(a, b)| a + b)
            .collect();
            
        Self {
            dimensions: self.dimensions,
            values,
        }
    }
}

impl Sub for Vector {
    type Output = Self;
    
    fn sub(self, other: Self) -> Self::Output {
        assert_eq!(self.dimensions, other.dimensions, "Vectors must have the same dimensions");
        
        let values = self.values
            .iter()
            .zip(other.values.iter())
            .map(|(a, b)| a - b)
            .collect();
            
        Self {
            dimensions: self.dimensions,
            values,
        }
    }
}

impl Mul<f32> for Vector {
    type Output = Self;
    
    fn mul(self, scalar: f32) -> Self::Output {
        let values = self.values.iter().map(|v| v * scalar).collect();
        
        Self {
            dimensions: self.dimensions,
            values,
        }
    }
}

impl Div<f32> for Vector {
    type Output = Self;
    
    fn div(self, scalar: f32) -> Self::Output {
        assert!(scalar != 0.0, "Cannot divide by zero");
        
        let values = self.values.iter().map(|v| v / scalar).collect();
        
        Self {
            dimensions: self.dimensions,
            values,
        }
    }
}