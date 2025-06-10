use digest::Digest;

#[cfg(feature = "sha2")]
use sha2::Sha256;
#[cfg(feature = "sha3")]
use sha3::Sha3_256;
#[cfg(feature = "blake3")]
use blake3::Hasher as Blake3;

/// Extension trait providing a convenience `hash_bytes` method for digest
/// implementations.
pub trait DigestExt: Digest + Default {
    /// Hash the provided bytes and return the digest bytes.
    fn hash_bytes(bytes: &[u8]) -> Vec<u8> {
        let mut hasher = Self::new();
        hasher.update(bytes);
        hasher.finalize().to_vec()
    }
}

impl<T> DigestExt for T where T: Digest + Default {}

/// Hash bytes using the default algorithm (SHA-256).
#[cfg(feature = "sha3")]
pub fn default_hash_bytes(bytes: &[u8]) -> Vec<u8> {
    Sha3_256::hash_bytes(bytes)
}

#[cfg(feature = "blake3")]
pub fn default_hash_bytes(bytes: &[u8]) -> Vec<u8> {
    let mut hasher = Blake3::new();
    hasher.update(bytes);
    hasher.finalize().as_bytes().to_vec()
}

#[cfg(all(not(feature = "sha3"), not(feature = "blake3")))]
pub fn default_hash_bytes(bytes: &[u8]) -> Vec<u8> {
    Sha256::hash_bytes(bytes)
}
