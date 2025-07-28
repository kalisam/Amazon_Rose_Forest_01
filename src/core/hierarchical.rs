use crate::core::vector::Vector;

#[derive(Debug, Clone)]
pub struct Cluster {
    pub centroid: Vector,
    pub members: Vec<Vector>,
}

impl Cluster {
    fn new(vector: Vector) -> Self {
        Self {
            centroid: vector.clone(),
            members: vec![vector],
        }
    }

    fn recompute_centroid(&mut self) {
        if self.members.is_empty() {
            return;
        }
        let dimensions = self.members[0].dimensions;
        let mut sums = vec![0.0f32; dimensions];
        for v in &self.members {
            for (i, val) in v.values.iter().enumerate() {
                sums[i] += val;
            }
        }
        let len_inv = 1.0 / self.members.len() as f32;
        for sum in &mut sums {
            *sum *= len_inv;
        }
        self.centroid = Vector::new(sums);
    }
}

/// Perform a simple agglomerative clustering using Euclidean distance.
/// Clusters are merged until the closest pair has distance greater than
/// `threshold`.
pub fn cluster_vectors(vectors: &[Vector], threshold: f32) -> Vec<Cluster> {
    let mut clusters: Vec<Cluster> = vectors.iter().cloned().map(Cluster::new).collect();
    if clusters.is_empty() {
        return clusters;
    }
    loop {
        let mut best_dist = f32::MAX;
        let mut best_pair: Option<(usize, usize)> = None;
        for i in 0..clusters.len() {
            for j in (i + 1)..clusters.len() {
                let dist = clusters[i]
                    .centroid
                    .euclidean_distance(&clusters[j].centroid);
                if dist < best_dist {
                    best_dist = dist;
                    best_pair = Some((i, j));
                }
            }
        }
        match best_pair {
            Some((i, j)) if best_dist <= threshold => {
                let mut members = clusters[i].members.clone();
                members.extend(clusters[j].members.clone());
                clusters[i].members = members;
                clusters[i].recompute_centroid();
                clusters.remove(j);
            }
            _ => break,
        }
    }
    clusters
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_clustering() {
        let v1 = Vector::new(vec![0.0, 0.0]);
        let v2 = Vector::new(vec![0.1, -0.1]);
        let v3 = Vector::new(vec![5.0, 5.0]);
        let v4 = Vector::new(vec![5.2, 4.9]);
        let clusters = cluster_vectors(&[v1, v2, v3, v4], 0.5);
        assert_eq!(clusters.len(), 2);
    }

    #[test]
    fn test_single_cluster_when_threshold_large() {
        let vectors = vec![
            Vector::new(vec![0.0, 0.0]),
            Vector::new(vec![1.0, 0.0]),
            Vector::new(vec![0.0, 1.0]),
        ];
        let clusters = cluster_vectors(&vectors, 10.0);
        assert_eq!(clusters.len(), 1);
        assert_eq!(clusters[0].members.len(), 3);
    }

    #[test]
    fn test_no_merge_when_threshold_small() {
        let vectors = vec![Vector::new(vec![0.0, 0.0]), Vector::new(vec![1.0, 1.0])];
        let clusters = cluster_vectors(&vectors, 0.1);
        assert_eq!(clusters.len(), 2);
    }
}
