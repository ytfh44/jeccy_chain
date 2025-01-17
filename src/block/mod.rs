//! Data structures and operations related to blockchain blocks
//! This module provides the core block-related types and functions for creating and managing blocks,
//! headers, and transactions in the blockchain.

use sp_runtime::{
    generic,
    traits::{Block as BlockT, Header as HeaderT, NumberFor},
    Digest, DigestItem,
};
use sp_runtime::OpaqueExtrinsic;
use serde::{Serialize, Deserialize};
use sp_core::H256;
use parity_scale_codec::{Encode, Decode};
use scale_info::TypeInfo;

/// Transaction data structure that represents a single blockchain transaction call
#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode, TypeInfo, PartialEq, Eq)]
pub struct Call {
    /// Raw transaction data as bytes
    pub data: Vec<u8>,
}

/// Transaction type alias representing an opaque extrinsic
pub type Transaction = OpaqueExtrinsic;

/// Block header type with 64-bit block number and Blake2-256 hashing
pub type Header = generic::Header<u64, sp_runtime::traits::BlakeTwo256>;

/// Complete block type composed of a header and transactions
pub type Block = generic::Block<Header, Transaction>;

/// Creates a new block header with the specified parameters
pub fn create_header(
    number: u64,
    parent_hash: H256,
    state_root: H256,
    extrinsics_root: H256,
    digest: Digest,
) -> Header {
    generic::Header::new(
        number,
        extrinsics_root,
        state_root,
        parent_hash,
        digest,
    )
}

/// Creates a new block from a header and list of transactions
pub fn create_block(
    header: Header,
    transactions: Vec<Transaction>,
) -> Block {
    generic::Block::new(header, transactions)
}

/// Creates a new transaction from a Call struct
pub fn create_transaction(call: Call) -> Transaction {
    OpaqueExtrinsic::from_bytes(&call.encode())
        .expect("Call encoding should be valid")
}