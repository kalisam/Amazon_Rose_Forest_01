// Hilbert curve implementation for efficient mapping between
// multidimensional vectors and one-dimensional space

#[derive(Debug, Clone)]
pub struct HilbertCurve {
    dimensions: usize,
    bits_per_dimension: usize,
}

impl HilbertCurve {
    /// Create a new Hilbert curve with the specified number of dimensions
    /// and bits per dimension.
    pub fn new(dimensions: usize, bits_per_dimension: usize) -> Self {
        assert!(dimensions > 0, "Dimensions must be greater than zero");
        assert!(bits_per_dimension > 0, "Bits per dimension must be greater than zero");
        assert!(dimensions * bits_per_dimension <= 64, "Total bits must fit in a u64");
        
        Self {
            dimensions,
            bits_per_dimension,
        }
    }

    /// Get the bits per dimension value
    pub fn bits_per_dimension(&self) -> usize {
        self.bits_per_dimension
    }
    
    /// Convert a multidimensional point to its Hilbert index
    pub fn point_to_index(&self, point: &[u64]) -> u64 {
        assert_eq!(point.len(), self.dimensions, "Point dimensions don't match curve dimensions");
        
        // Validate point coordinates are within range
        for &p in point {
            assert!(p < (1 << self.bits_per_dimension), "Coordinate exceeds maximum for bits_per_dimension");
        }
        
        let mut index: u64 = 0;
        let max_bit = 1 << (self.bits_per_dimension - 1);
        
        // For each bit position, from most significant to least significant
        for bit in (0..self.bits_per_dimension).rev() {
            let bit_mask = 1 << bit;
            let mut current_bits = 0;
            
            // Extract the bit from each dimension
            for dim in 0..self.dimensions {
                if (point[dim] & bit_mask) != 0 {
                    current_bits |= 1 << dim;
                }
            }
            
            // Interleave the bits into the result
            index = (index << self.dimensions) | self.transform_bits(current_bits, self.dimensions) as u64;
        }
        
        index
    }
    
    /// Convert a Hilbert index back to its multidimensional point
    pub fn index_to_point(&self, mut index: u64) -> Vec<u64> {
        let mut point = vec![0; self.dimensions];
        
        // For each bit position, from least significant to most significant
        for bit in 0..self.bits_per_dimension {
            // Extract the bits for the current level
            let current_bits = index & ((1 << self.dimensions) - 1);
            index >>= self.dimensions;
            
            // Transform the bits back to original ordering
            let transformed_bits = self.inverse_transform_bits(current_bits as usize, self.dimensions);
            
            // Set the appropriate bit in each dimension
            for dim in 0..self.dimensions {
                if (transformed_bits & (1 << dim)) != 0 {
                    point[dim] |= 1 << bit;
                }
            }
        }
        
        point
    }
    
    /// Transform bits according to Hilbert curve rules
    fn transform_bits(&self, bits: usize, num_bits: usize) -> usize {
        let mut transformed = bits;
        let mut temp;
        
        // Apply Gray code transformation
        transformed ^= transformed >> 1;
        
        // Additional bit manipulations for higher dimensions
        // This is a simplified implementation for common dimensions
        if num_bits >= 2 {
            // Common transformations for 2D and above
            temp = (transformed >> 1) & 1;
            transformed ^= (bits & 1) << 1;
            transformed ^= temp;
        }
        
        transformed
    }
    
    /// Inverse transform bits to recover original position
    fn inverse_transform_bits(&self, bits: usize, num_bits: usize) -> usize {
        let mut transformed = bits;
        let mut temp;
        
        // Undo the bit manipulations for higher dimensions
        if num_bits >= 2 {
            temp = (transformed >> 1) & 1;
            transformed ^= temp;
            transformed ^= (bits & 2) >> 1;
        }
        
        // Undo Gray code transformation
        let mut mask = bits;
        while mask != 0 {
            mask >>= 1;
            transformed ^= mask;
        }
        
        transformed
    }
    
    /// Calculate the distance between two points along the Hilbert curve
    pub fn distance(&self, point1: &[u64], point2: &[u64]) -> u64 {
        let index1 = self.point_to_index(point1);
        let index2 = self.point_to_index(point2);
        
        if index1 > index2 {
            index1 - index2
        } else {
            index2 - index1
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_2d_hilbert_curve() {
        let curve = HilbertCurve::new(2, 3);  // 2D, 3 bits per dimension
        
        // Test some known 2D mappings
        let test_points = [
            // point, expected index
            (vec![0, 0], 0),
            (vec![0, 1], 1),
            (vec![1, 1], 2),
            (vec![1, 0], 3),
            (vec![2, 0], 4),
            (vec![3, 0], 5),
            (vec![3, 1], 6),
            (vec![2, 1], 7),
        ];
        
        for (point, expected) in &test_points {
            let index = curve.point_to_index(point);
            assert_eq!(index, *expected, "Point {:?} should map to index {}", point, expected);
            
            let restored = curve.index_to_point(index);
            assert_eq!(&restored, point, "Index {} should map back to point {:?}", index, point);
        }
    }
    
    #[test]
    fn test_distance() {
        let curve = HilbertCurve::new(2, 3);  // 2D, 3 bits per dimension
        
        let point1 = vec![0, 0];
        let point2 = vec![1, 1];
        let point3 = vec![7, 7];
        
        assert_eq!(curve.distance(&point1, &point2), 2);
        assert_eq!(curve.distance(&point1, &point3), 63);
    }
}