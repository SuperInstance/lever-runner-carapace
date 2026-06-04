//! BLAKE2b state hashing — deterministic across platforms.

use blake2::{Blake2b, Digest};
use std::fmt;

/// 32-byte hash digest for intent strings.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct IntentHash([u8; 32]);

impl IntentHash {
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    pub fn to_hex(&self) -> String {
        self.0.iter().map(|b| format!("{:02x}", b)).collect()
    }
}

impl fmt::Debug for IntentHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "IntentHash({})", self.to_hex())
    }
}

/// Hash an intent string with BLAKE2b-256.
///
/// Deterministic: same input always produces same output on all platforms.
pub fn hash_intent(intent: &str) -> IntentHash {
    use blake2::digest::consts::U32;
    let mut hasher = Blake2b::<U32>::new();
    hasher.update(intent.as_bytes());
    let result = hasher.finalize();
    let mut out = [0u8; 32];
    out.copy_from_slice(&result);
    IntentHash(out)
}

/// Hash an intent with a context salt for domain separation.
pub fn hash_intent_with_salt(intent: &str, salt: &[u8]) -> IntentHash {
    use blake2::digest::consts::U32;
    let mut hasher = Blake2b::<U32>::new();
    hasher.update(salt);
    hasher.update(intent.as_bytes());
    let result = hasher.finalize();
    let mut out = [0u8; 32];
    out.copy_from_slice(&result);
    IntentHash(out)
}
