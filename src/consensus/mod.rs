//! Consensus mechanism related traits and implementations

use async_trait::async_trait;
use crate::{block::{self, Block}, Error, Result};
use sp_core::{sr25519, Pair, crypto::ByteArray};
use sp_runtime::{
    traits::{IdentifyAccount, Header as HeaderT, Verify},
    DigestItem as RuntimeDigestItem,
    generic::DigestItem,
    OpaqueExtrinsic,
    Digest,
    traits::BlakeTwo256,
    BoundedVec
};
use parity_scale_codec::{Encode, Decode};
use sp_core::H256;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock, PoisonError};
use std::time::{SystemTime, UNIX_EPOCH};

type AccountId = <sr25519::Public as IdentifyAccount>::AccountId;
type Signature = sr25519::Signature;
type BlockHeader = sp_runtime::generic::Header<u64, BlakeTwo256>;

#[derive(Debug, thiserror::Error)]
pub enum ConsensusError {
    #[error("Lock error: {0}")]
    LockError(String),
    #[error("Time error: {0}")]
    TimeError(String),
    #[error("Invalid proposer: {0}")]
    InvalidProposer(String),
    #[error("Not current proposer: {0}")]
    NotCurrentProposer(String),
    #[error("Invalid signature: {0}")]
    InvalidSignature(String),
    #[error("Block finalization failed: {0}")]
    FinalizationFailed(String),
}

// Error conversion implementation
impl From<ConsensusError> for Error {
    fn from(err: ConsensusError) -> Self {
        Error::Consensus(err.to_string())
    }
}

impl<T> From<PoisonError<T>> for ConsensusError {
    fn from(err: PoisonError<T>) -> Self {
        ConsensusError::LockError(err.to_string())
    }
}

/// Consensus mechanism trait
#[async_trait]
pub trait Consensus: Send + Sync {
    /// Validate a block
    async fn validate_block(&self, block: &Block) -> Result<()>;
    
    /// Generate a new block
    async fn generate_block(&self, transactions: Vec<Vec<u8>>) -> Result<Block>;
    
    /// Finalize a block
    async fn finalize_block(&self, block: &Block) -> Result<()>;
}

/// Validator information
#[derive(Clone, Debug)]
pub struct Validator {
    /// Validator's public key
    pub public_key: sr25519::Public,
    /// Amount of staked tokens
    pub stake: u64,
    /// Number of votes received
    pub votes: u64,
}

/// Block confirmation status
#[derive(Clone, Debug, Default)]
struct BlockConfirmation {
    /// Set of validators who have confirmed the block
    confirmations: HashSet<AccountId>,
    /// Whether the block has been finalized
    finalized: bool,
}

/// DPoS consensus implementation
pub struct DPoS {
    /// Current list of validators
    validators: Arc<RwLock<Vec<Validator>>>,
    /// Voting records mapping candidates to their voters
    votes: Arc<RwLock<HashMap<AccountId, HashSet<AccountId>>>>,
    /// Maximum number of validators
    validator_count: usize,
    /// Duration of each round in seconds
    round_interval: u64,
    /// Node's keypair for signing
    key_pair: sr25519::Pair,
    /// Block confirmation status tracking
    confirmations: Arc<RwLock<HashMap<H256, BlockConfirmation>>>,
}

impl DPoS {
    /// Create a new DPoS instance
    pub fn new(key_pair: sr25519::Pair, validator_count: usize, round_interval: u64) -> Self {
        Self {
            validators: Arc::new(RwLock::new(Vec::new())),
            votes: Arc::new(RwLock::new(HashMap::new())),
            validator_count,
            round_interval,
            key_pair,
            confirmations: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Add a new validator to the set
    pub fn add_validator(&self, validator: Validator) -> Result<()> {
        let mut validators = self.validators.write().map_err(|e| Error::from(ConsensusError::from(e)))?;
        validators.push(validator);
        Ok(())
    }

    /// Cast a vote from a voter to a candidate validator
    pub fn vote(&self, voter: AccountId, candidate: AccountId) -> Result<()> {
        let mut votes = self.votes.write().map_err(|e| Error::from(ConsensusError::from(e)))?;
        votes.entry(candidate)
            .or_insert_with(HashSet::new)
            .insert(voter);
        Ok(())
    }

    /// Update the validator set based on votes and stakes
    fn update_validators(&self) -> Result<()> {
        let votes = self.votes.read().map_err(|e| Error::from(ConsensusError::from(e)))?;
        let mut validators = self.validators.write().map_err(|e| Error::from(ConsensusError::from(e)))?;
        
        // Update vote counts for each validator
        for validator in validators.iter_mut() {
            let key = validator.public_key.into_account();
            validator.votes = votes.get(&key)
                .map(|voters| voters.len() as u64)
                .unwrap_or(0);
        }

        // Sort validators by stake and vote count
        validators.sort_by(|a, b| {
            let a_score = a.stake * a.votes;
            let b_score = b.stake * b.votes;
            b_score.cmp(&a_score)
        });

        // Keep only the top N validators
        validators.truncate(self.validator_count);
        
        Ok(())
    }

    /// Check if this node is the current round's block proposer
    fn is_current_proposer(&self) -> Result<bool> {
        let validators = self.validators.read().map_err(|e| Error::from(ConsensusError::from(e)))?;
        
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| ConsensusError::TimeError(e.to_string()))?
            .as_secs();
        
        let round = timestamp / self.round_interval;
        let proposer_index = (round as usize) % validators.len();
        
        Ok(validators.get(proposer_index)
            .map(|v| v.public_key == self.key_pair.public())
            .unwrap_or(false))
    }

    /// Confirm a block by a validator and check for finalization
    pub fn confirm_block(&self, block_hash: H256, validator: AccountId) -> Result<bool> {
        let mut confirmations = self.confirmations.write().map_err(|e| Error::from(ConsensusError::from(e)))?;
        let confirmation = confirmations.entry(block_hash).or_default();
        
        if confirmation.finalized {
            return Ok(true);
        }

        confirmation.confirmations.insert(validator);
        
        let validators = self.validators.read().map_err(|e| Error::from(ConsensusError::from(e)))?;
        let threshold = validators.len() * 2 / 3;
        
        if confirmation.confirmations.len() > threshold {
            confirmation.finalized = true;
            return Ok(true);
        }
        
        Ok(false)
    }

    /// Sign a block header with the node's private key
    fn sign_header(&self, header: &BlockHeader) -> Signature {
        let encoded = header.encode();
        self.key_pair.sign(&encoded)
    }
}

#[async_trait]
impl Consensus for DPoS {
    async fn validate_block(&self, block: &Block) -> Result<()> {
        // Validate block signature
        let validators = self.validators.read().map_err(|e| Error::from(ConsensusError::from(e)))?;

        // Extract author information and signature from block
        let mut author = None;
        let mut signature = None;
        
        for log in block.header.digest().logs() {
            match log {
                DigestItem::PreRuntime(_, data) => {
                    author = Some(data);
                }
                DigestItem::Seal(_, sig) => {
                    signature = Some(sig);
                }
                _ => {}
            }
        }

        let author = author.ok_or_else(|| ConsensusError::InvalidProposer("Block author not found".to_string()))?;

        // Verify that the author is a valid validator
        let validator = validators.iter()
            .find(|v| v.public_key.to_raw_vec() == *author)
            .ok_or_else(|| ConsensusError::InvalidProposer("Invalid block proposer".to_string()))?;

        // Verify block signature
        if let Some(sig_bytes) = signature {
            let sig = Signature::from_slice(&sig_bytes)
                .map_err(|_| ConsensusError::InvalidSignature("Invalid signature format".to_string()))?;
            
            let encoded = block.header.encode();
            if !sig.verify(encoded.as_slice(), &validator.public_key) {
                return Err(ConsensusError::InvalidSignature("Signature verification failed".to_string()).into());
            }
        }
        
        Ok(())
    }
    
    async fn generate_block(&self, transactions: Vec<Vec<u8>>) -> Result<Block> {
        // Verify current node is the designated block proposer
        if !self.is_current_proposer()? {
            return Err(ConsensusError::NotCurrentProposer("Not the current proposer".to_string()).into());
        }
        
        // Update validator set
        self.update_validators()?;
        
        // Convert transactions to block extrinsics format
        let extrinsics: Vec<OpaqueExtrinsic> = transactions.into_iter()
            .map(|tx| OpaqueExtrinsic::from_bytes(&tx).expect("Valid transaction bytes"))
            .collect();

        // Create block header
        let parent_hash = H256::default(); // TODO: Get parent block hash
        let number = 0; // TODO: Get block height
        let state_root = H256::default(); // TODO: Calculate state root
        let extrinsics_root = H256::default(); // TODO: Calculate extrinsics root

        // Create block header digest
        let mut digest = Digest::default();
        
        // Add pre-runtime information (author)
        let author_bytes = self.key_pair.public().to_raw_vec();
        let author_id = [0u8; 4];
        digest.push(DigestItem::PreRuntime(
            author_id,
            author_bytes
        ));

        // Create block header
        let mut header = block::create_header(
            number,
            parent_hash,
            state_root,
            extrinsics_root,
            digest,
        );

        // Sign block header
        let signature = self.sign_header(&header);
        
        // Add signature to block header
        let sig_bytes = signature.to_raw_vec();
        let seal_id = [0u8; 4];
        header.digest_mut().push(DigestItem::Seal(
            seal_id,
            sig_bytes
        ));

        // Create complete block
        Ok(block::create_block(header, extrinsics))
    }
    
    async fn finalize_block(&self, block: &Block) -> Result<()> {
        // Get block hash
        let block_hash = block.header.hash();
        
        // Get current validator ID
        let validator_id = self.key_pair.public().into_account();
        
        // Confirm block and check finalization
        if self.confirm_block(block_hash, validator_id)? {
            Ok(())
        } else {
            Err(ConsensusError::FinalizationFailed("Block not finalized".to_string()).into())
        }
    }
} 
