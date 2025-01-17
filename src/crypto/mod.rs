//! Cryptography-related functionality module

use sp_core::{crypto::{Pair, Public}, sr25519};

/// Key pair
pub type KeyPair = sr25519::Pair;
/// Public key
pub type PublicKey = sr25519::Public;
/// Signature
pub type Signature = sr25519::Signature;

/// Calculate hash value
pub fn hash(data: &[u8]) -> [u8; 32] {
    use sp_core::hashing::blake2_256;
    blake2_256(data)
}

/// Generate a new key pair
pub fn generate_keypair() -> KeyPair {
    KeyPair::generate().0
}

/// Generate key pair from seed
pub fn keypair_from_seed(seed: &[u8]) -> Result<KeyPair, sp_core::crypto::SecretStringError> {
    KeyPair::from_seed_slice(seed)
}

/// Sign data
pub fn sign(keypair: &KeyPair, message: &[u8]) -> Signature {
    keypair.sign(message)
}

/// Verify signature
pub fn verify(
    signature: &Signature,
    message: &[u8],
    public: &PublicKey,
) -> bool {
    sr25519::Pair::verify(signature, message, public)
} 