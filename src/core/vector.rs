use serde::{Deserialize, Serialize};
use std::ops::{Add, Div, Mul, Sub};
use std::simd::{f32x4, mask32x4, StdFloat};

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
    
    pub fn ones(dimensions: usize) -> Self {
        Self {
            dimensions,
            values: vec![1.0; dimensions],
        }
    }
    
    pub fn random(dimensions: usize) -> Self {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let values = (0..dimensions).map(|_| rng.gen::<f32>()).collect();
        Self { dimensions, values }
    }
    
    pub fn random_normal(dimensions: usize, mean: f32, std_dev: f32) -> Self {
        use rand_distr::{Distribution, Normal};
        let normal = Normal::new(mean as f64, std_dev as f64).unwrap();
        let mut rng = rand::thread_rng();
        let values = (0..dimensions)
            .map(|_| normal.sample(&mut rng) as f32)
            .collect();
        Self { dimensions, values }
    }
    
    pub fn dot(&self, other: &Vector) -> f32 {
        assert_eq!(self.dimensions, other.dimensions, "Vectors must have the same dimensions");
        
        // Use SIMD acceleration for vectors with dimensions divisible by 4
        if self.dimensions % 4 == 0 {
            self.dot_simd(other)
        } else {
            self.dot_scalar(other)
        }
    }
    
    fn dot_scalar(&self, other: &Vector) -> f32 {
        self.values.iter().zip(other.values.iter()).map(|(a, b)| a * b).sum()
    }
    
    fn dot_simd(&self, other: &Vector) -> f32 {
        let chunks = self.dimensions / 4;
        let mut sum = f32x4::splat(0.0);
        
        for i in 0..chunks {
            let start = i * 4;
            let a = f32x4::from_slice(&self.values[start..start + 4]);
            let b = f32x4::from_slice(&other.values[start..start + 4]);
            sum += a * b;
        }
        
        sum.horizontal_sum()
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
        
        // Use SIMD acceleration for vectors with dimensions divisible by 4
        if self.dimensions % 4 == 0 {
            self.euclidean_distance_simd(other)
        } else {
            self.euclidean_distance_scalar(other)
        }
    }
    
    fn euclidean_distance_scalar(&self, other: &Vector) -> f32 {
        let sum_squared_diff: f32 = self.values
            .iter()
            .zip(other.values.iter())
            .map(|(a, b)| (a - b).powi(2))
            .sum();
            
        sum_squared_diff.sqrt()
    }
    
    fn euclidean_distance_simd(&self, other: &Vector) -> f32 {
        let chunks = self.dimensions / 4;
        let mut sum = f32x4::splat(0.0);
        
        for i in 0..chunks {
            let start = i * 4;
            let a = f32x4::from_slice(&self.values[start..start + 4]);
            let b = f32x4::from_slice(&other.values[start..start + 4]);
            let diff = a - b;
            sum += diff * diff;
        }
        
        sum.horizontal_sum().sqrt()
    }
    
    pub fn manhattan_distance(&self, other: &Vector) -> f32 {
        assert_eq!(self.dimensions, other.dimensions, "Vectors must have the same dimensions");
        
        self.values
            .iter()
            .zip(other.values.iter())
            .map(|(a, b)| (a - b).abs())
            .sum()
    }
    
    pub fn hamming_distance(&self, other: &Vector) -> usize {
        assert_eq!(self.dimensions, other.dimensions, "Vectors must have the same dimensions");
        
        self.values
            .iter()
            .zip(other.values.iter())
            .filter(|(&a, &b)| (a - b).abs() > f32::EPSILON)
            .count()
    }
    
    pub fn batch_process<F>(&self, others: &[Vector], f: F) -> Vec<f32>
    where
        F: Fn(&Vector, &Vector) -> f32,
    {
        others.iter().map(|other| f(self, other)).collect()
    }
    
    pub fn batch_cosine_similarity(&self, others: &[Vector]) -> Vec<f32> {
        self.batch_process(others, |a, b| a.cosine_similarity(b))
    }
    
    pub fn batch_euclidean_distance(&self, others: &[Vector]) -> Vec<f32> {
        self.batch_process(others, |a, b| a.euclidean_distance(b))
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

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_vector_operations() {
        let v1 = Vector::new(vec![1.0, 2.0, 3.0]);
        let v2 = Vector::new(vec![4.0, 5.0, 6.0]);
        
        // Test addition
        let v_sum = v1.clone() + v2.clone();
        assert_eq!(v_sum.values, vec![5.0, 7.0, 9.0]);
        
        // Test subtraction
        let v_diff = v2.clone() - v1.clone();
        assert_eq!(v_diff.values, vec![3.0, 3.0, 3.0]);
        
        // Test scalar multiplication
        let v_scaled = v1.clone() * 2.0;
        assert_eq!(v_scaled.values, vec![2.0, 4.0, 6.0]);
        
        // Test dot product
        let dot = v1.dot(&v2);
        assert_eq!(dot, 1.0 * 4.0 + 2.0 * 5.0 + 3.0 * 6.0);
        
        // Test magnitude
        let mag = v1.magnitude();
        assert!((mag - (1.0f32 + 4.0 + 9.0).sqrt()).abs() < 1e-6);
        
        // Test normalization
        let v_norm = v1.normalize();
        let expected_mag = 1.0;
        assert!((v_norm.magnitude() - expected_mag).abs() < 1e-6);
    }
    
    #[test]
    fn test_distance_metrics() {
        let v1 = Vector::new(vec![1.0, 2.0, 3.0]);
        let v2 = Vector::new(vec![4.0, 5.0, 6.0]);
        
        // Test Euclidean distance
        let euclidean = v1.euclidean_distance(&v2);
        assert!((euclidean - ((3.0 * 3.0 + 3.0 * 3.0 + 3.0 * 3.0) as f32).sqrt()).abs() < 1e-6);
        
        // Test Manhattan distance
        let manhattan = v1.manhattan_distance(&v2);
        assert_eq!(manhattan, 9.0);
        
        // Test Hamming distance
        let hamming = v1.hamming_distance(&v2);
        assert_eq!(hamming, 3);
        
        // Test cosine similarity
        let cosine = v1.cosine_similarity(&v2);
        let expected = (1.0 * 4.0 + 2.0 * 5.0 + 3.0 * 6.0) / 
                      ((1.0 * 1.0 + 2.0 * 2.0 + 3.0 * 3.0).sqrt() * 
                       (4.0 * 4.0 + 5.0 * 5.0 + 6.0 * 6.0).sqrt());
        assert!((cosine - expected).abs() < 1e-6);
    }
    
    #[test]
    fn test_batch_processing() {
        let v1 = Vector::new(vec![1.0, 2.0, 3.0]);
        let others = vec![
            Vector::new(vec![4.0, 5.0, 6.0]),
            Vector::new(vec![7.0, 8.0, 9.0]),
        ];
        
        let distances = v1.batch_euclidean_distance(&others);
        assert_eq!(distances.len(), 2);
        
        let similarities = v1.batch_cosine_similarity(&others);
        assert_eq!(similarities.len(), 2);
    }
}