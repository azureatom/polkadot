// Copyright 2017 Parity Technologies (UK) Ltd.
// This file is part of Polkadot.

// Polkadot is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Polkadot is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Polkadot.  If not, see <http://www.gnu.org/licenses/>.

//! Strongly typed API for Polkadot based around the locally-compiled native
//! runtime.

extern crate polkadot_executor;
extern crate polkadot_primitives as primitives;
extern crate polkadot_runtime as runtime;
extern crate substrate_codec as codec;
extern crate substrate_runtime_io as runtime_io;
extern crate substrate_client as client;
extern crate substrate_executor as substrate_executor;
extern crate substrate_runtime_executive;
extern crate substrate_primitives;
extern crate substrate_runtime_primitives as runtime_primitives;
extern crate substrate_state_machine as state_machine;

#[macro_use]
extern crate error_chain;

#[cfg(test)]
extern crate substrate_keyring as keyring;

pub mod full;
pub mod light;

use primitives::{AccountId, Block, BlockId, Hash, Index, SessionKey, Timestamp,
	UncheckedExtrinsic};
use runtime::Address;
use primitives::parachain::{CandidateReceipt, DutyRoster, Id as ParaId};

error_chain! {
	errors {
		/// Unknown runtime code.
		UnknownRuntime {
			description("Unknown runtime code")
			display("Unknown runtime code")
		}
		/// Unknown block ID.
		UnknownBlock(b: String) {
			description("Unknown block")
			display("Unknown block {}", b)
		}
		/// Some other error.
		// TODO: allow to be specified as associated type of PolkadotApi
		Other(e: Box<::std::error::Error + Send>) {
			description("Other error")
			display("Other error: {}", e.description())
		}
	}

	links {
		Executor(substrate_executor::error::Error, substrate_executor::error::ErrorKind);
	}
}

impl From<client::error::Error> for Error {
	fn from(e: client::error::Error) -> Error {
		match e {
			client::error::Error(client::error::ErrorKind::UnknownBlock(b), _) => Error::from_kind(ErrorKind::UnknownBlock(b)),
			other => Error::from_kind(ErrorKind::Other(Box::new(other) as Box<_>)),
		}
	}
}

/// A checked block identifier.
pub trait CheckedBlockId: Clone + 'static {
	/// Yield the underlying block ID.
	fn block_id(&self) -> &BlockId;
}

/// Build new blocks.
pub trait BlockBuilder {
	/// Push an extrinsic onto the block. Fails if the extrinsic is invalid.
	fn push_extrinsic(&mut self, extrinsic: UncheckedExtrinsic) -> Result<()>;

	/// Bake the block with provided extrinsics.
	fn bake(self) -> Result<Block>;
}

/// Trait encapsulating the Polkadot API.
///
/// All calls should fail when the exact runtime is unknown.
pub trait PolkadotApi {
	/// A checked block ID. Used to avoid redundancy of code check.
	type CheckedBlockId: CheckedBlockId;
	/// The block builder for this API type.
	type BlockBuilder: BlockBuilder;

	/// Check whether requests at the given block ID can be served.
	///
	/// It should not be possible to instantiate this type without going
	/// through this function.
	fn check_id(&self, id: BlockId) -> Result<Self::CheckedBlockId>;

	/// Get session keys at a given block.
	fn session_keys(&self, at: &Self::CheckedBlockId) -> Result<Vec<SessionKey>>;

	/// Get validators at a given block.
	fn validators(&self, at: &Self::CheckedBlockId) -> Result<Vec<AccountId>>;

	/// Get the value of the randomness beacon at a given block.
	fn random_seed(&self, at: &Self::CheckedBlockId) -> Result<Hash>;

	/// Get the authority duty roster at a block.
	fn duty_roster(&self, at: &Self::CheckedBlockId) -> Result<DutyRoster>;

	/// Get the timestamp registered at a block.
	fn timestamp(&self, at: &Self::CheckedBlockId) -> Result<Timestamp>;

	/// Get the nonce (né index) of an account at a block.
	fn index(&self, at: &Self::CheckedBlockId, account: AccountId) -> Result<Index>;

	/// Get the account id of an address at a block.
	fn lookup(&self, at: &Self::CheckedBlockId, address: Address) -> Result<Option<AccountId>>;

	/// Get the active parachains at a block.
	fn active_parachains(&self, at: &Self::CheckedBlockId) -> Result<Vec<ParaId>>;

	/// Get the validation code of a parachain at a block. If the parachain is active, this will always return `Some`.
	fn parachain_code(&self, at: &Self::CheckedBlockId, parachain: ParaId) -> Result<Option<Vec<u8>>>;

	/// Get the chain head of a parachain. If the parachain is active, this will always return `Some`.
	fn parachain_head(&self, at: &Self::CheckedBlockId, parachain: ParaId) -> Result<Option<Vec<u8>>>;

	/// Evaluate a block. Returns true if the block is good, false if it is known to be bad,
	/// and an error if we can't evaluate for some reason.
	fn evaluate_block(&self, at: &Self::CheckedBlockId, block: Block) -> Result<bool>;

	/// Build a block on top of the given, with inherent extrinsics pre-pushed.
	fn build_block(&self, at: &Self::CheckedBlockId, timestamp: Timestamp, new_heads: Vec<CandidateReceipt>) -> Result<Self::BlockBuilder>;

	/// Attempt to produce the (encoded) inherent extrinsics for a block being built upon the given.
	/// This may vary by runtime and will fail if a runtime doesn't follow the same API.
	fn inherent_extrinsics(&self, at: &Self::CheckedBlockId, timestamp: Timestamp, new_heads: Vec<CandidateReceipt>) -> Result<Vec<UncheckedExtrinsic>>;
}

/// Mark for all Polkadot API implementations, that are making use of state data, stored locally.
pub trait LocalPolkadotApi: PolkadotApi {}

/// Mark for all Polkadot API implementations, that are fetching required state data from remote nodes.
pub trait RemotePolkadotApi: PolkadotApi {}
