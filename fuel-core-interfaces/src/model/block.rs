pub use super::BlockHeight;
use crate::{
    common::{
        fuel_crypto::Hasher,
        fuel_merkle::binary::in_memory::MerkleTree,
        fuel_tx::{
            Address,
            Bytes32,
            Transaction,
        },
        fuel_types::bytes::SerializableVec,
    },
    model::DaBlockHeight,
};
use chrono::{
    DateTime,
    TimeZone,
    Utc,
};
use core::ops::Deref;

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FuelBlockHeader {
    /// Fuel block height.
    pub height: BlockHeight,
    /// The layer 1 height of messages and events to include since the last layer 1 block number.
    /// This is not meant to represent the layer 1 block this was committed to. Validators will need
    /// to have some rules in place to ensure the block number was chosen in a reasonable way. For
    /// example, they should verify that the block number satisfies the finality requirements of the
    /// layer 1 chain. They should also verify that the block number isn't too stale and is increasing.
    /// Some similar concerns are noted in this issue: https://github.com/FuelLabs/fuel-specs/issues/220
    pub da_height: DaBlockHeight,
    /// Block header hash of the previous block.
    pub parent_hash: Bytes32,
    /// Merkle root of all previous block header hashes.
    pub prev_root: Bytes32,
    /// Merkle root of transactions.
    pub transactions_root: Bytes32,
    /// The block producer time
    pub time: DateTime<Utc>,
    /// The block producer public key
    pub producer: Address,
    /// Header Metadata
    #[cfg_attr(feature = "serde", serde(skip))]
    pub metadata: Option<HeaderMetadata>,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct HeaderMetadata {
    id: Bytes32,
}

impl FuelBlockHeader {
    pub fn recalculate_metadata(&mut self) {
        self.metadata = Some(HeaderMetadata { id: self.hash() });
    }
    fn hash(&self) -> Bytes32 {
        let mut hasher = Hasher::default();
        hasher.input(&self.height.to_bytes()[..]);
        hasher.input(&self.da_height.to_bytes()[..]);
        hasher.input(self.parent_hash.as_ref());
        hasher.input(self.prev_root.as_ref());
        hasher.input(self.transactions_root.as_ref());
        hasher.input(self.time.timestamp_millis().to_be_bytes());
        hasher.input(self.producer.as_ref());
        hasher.digest()
    }

    pub fn id(&self) -> Bytes32 {
        if let Some(ref metadata) = self.metadata {
            metadata.id
        } else {
            self.hash()
        }
    }

    pub fn transactions_root(txs: &[Transaction]) -> Bytes32 {
        let mut tree = MerkleTree::new();
        for tx in txs {
            // serialize tx into canonical format for hashing
            let ser_tx = tx.clone().to_bytes();
            tree.push(&ser_tx);
        }
        tree.root().into()
    }
}

impl Default for FuelBlockHeader {
    fn default() -> Self {
        Self {
            time: Utc.timestamp(0, 0),
            height: BlockHeight::default(),
            da_height: DaBlockHeight::default(),
            parent_hash: Bytes32::default(),
            prev_root: Bytes32::default(),
            transactions_root: Bytes32::default(),
            producer: Address::default(),
            metadata: None,
        }
    }
}

/// The compact representation of a block used in the database
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FuelBlockDb {
    pub header: FuelBlockHeader,
    pub transactions: Vec<Bytes32>,
}

impl FuelBlockDb {
    pub fn id(&self) -> Bytes32 {
        self.header.id()
    }
}

/// Fuel block with all transaction data included
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FuelBlock {
    pub header: FuelBlockHeader,
    pub transactions: Vec<Transaction>,
}

impl FuelBlock {
    pub fn id(&self) -> Bytes32 {
        self.header.id()
    }

    pub fn to_db_block(&self) -> FuelBlockDb {
        FuelBlockDb {
            header: self.header.clone(),
            transactions: self.transactions.iter().map(|tx| tx.id()).collect(),
        }
    }
}

/// This structure is created as placeholder for future usage.
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FuelBlockConsensus {}

#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SealedFuelBlock {
    pub block: FuelBlock,
    pub consensus: FuelBlockConsensus,
}

impl Deref for SealedFuelBlock {
    type Target = FuelBlock;

    fn deref(&self) -> &FuelBlock {
        &self.block
    }
}
