//! JeccyChain - 区块链底层支持库
//! 
//! 这个库提供了区块链相关的基础功能，包括：
//! - 区块和交易的基本数据结构
//! - 共识机制接口
//! - 状态转换和存储
//! - 密码学原语

#![warn(missing_docs)]
#![warn(rustdoc::missing_doc_code_examples)]

pub mod block;
pub mod consensus;
pub mod storage;
pub mod crypto;

/// 重新导出常用类型
pub mod prelude {
    pub use super::block::Block;
    pub use super::consensus::Consensus;
    pub use super::storage::Storage;
    pub use super::crypto::*;
}

/// 库的错误类型
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// 区块验证错误
    #[error("Block validation failed: {0}")]
    BlockValidation(String),
    
    /// 共识错误
    #[error("Consensus error: {0}")]
    Consensus(String),
    
    /// 存储错误
    #[error("Storage error: {0}")]
    Storage(String),
    
    /// 加密错误
    #[error("Cryptography error: {0}")]
    Crypto(String),
}

/// 库的结果类型
pub type Result<T> = std::result::Result<T, Error>;
