//! Storage-related traits and implementations for the blockchain
//! This module provides the core storage interfaces and memory-based implementation
//! for managing blocks in the blockchain system.

use async_trait::async_trait;
use crate::{block::Block, Result};
use sp_runtime::traits::{
    Header as HeaderT,
    Block as BlockT,
};

/// Storage trait defining the core storage operations for blockchain blocks
#[async_trait]
pub trait Storage: Send + Sync {
    /// Store a block in the storage
    async fn put_block(&mut self, block: Block) -> Result<()>;
    
    /// Retrieve a block by its hash
    async fn get_block(&self, hash: &[u8; 32]) -> Result<Option<Block>>;
    
    /// Get the most recently added block
    async fn get_latest_block(&self) -> Result<Option<Block>>;
    
    /// Get the current blockchain height
    async fn get_height(&self) -> Result<u64>;
    
    /// Check if a block with the given hash exists in storage
    async fn has_block(&self, hash: &[u8; 32]) -> Result<bool>;
}

/// In-memory storage implementation using a HashMap
pub struct MemoryStorage {
    /// Block mapping: hash -> block
    blocks: std::collections::HashMap<[u8; 32], Block>,
    /// Hash of the most recently added block
    latest_block_hash: Option<[u8; 32]>,
}

impl MemoryStorage {
    /// Create a new empty memory storage instance
    pub fn new() -> Self {
        Self {
            blocks: std::collections::HashMap::new(),
            latest_block_hash: None,
        }
    }
}

#[async_trait]
impl Storage for MemoryStorage {
    async fn put_block(&mut self, block: Block) -> Result<()> {
        let hash = BlockT::hash(&block);
        self.latest_block_hash = Some(hash.into());
        self.blocks.insert(hash.into(), block);
        Ok(())
    }
    
    async fn get_block(&self, hash: &[u8; 32]) -> Result<Option<Block>> {
        Ok(self.blocks.get(hash).cloned())
    }
    
    async fn get_latest_block(&self) -> Result<Option<Block>> {
        Ok(self.latest_block_hash
            .and_then(|hash| self.blocks.get(&hash))
            .cloned())
    }
    
    async fn get_height(&self) -> Result<u64> {
        Ok(self.latest_block_hash
            .and_then(|hash| self.blocks.get(&hash))
            .map(|block| *BlockT::header(block).number())
            .unwrap_or(0))
    }
    
    async fn has_block(&self, hash: &[u8; 32]) -> Result<bool> {
        Ok(self.blocks.contains_key(hash))
    }
}