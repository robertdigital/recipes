//! A Super Runtime. This runtime demonstrates most the recipe pallets in a single super runtime.

#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit = "256"]

// Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

use rstd::prelude::*;
use primitives::OpaqueMetadata;
use sp_runtime::{
	ApplyExtrinsicResult, transaction_validity::TransactionValidity, generic, create_runtime_str,
	impl_opaque_keys, MultiSignature
};
use sp_runtime::traits::{
    BlakeTwo256, Block as BlockT, ConvertInto, NumberFor, StaticLookup, Verify, IdentifyAccount
};
use sp_api::impl_runtime_apis;
use babe::SameAuthoritiesForever;
use grandpa::AuthorityList as GrandpaAuthorityList;
use grandpa::fg_primitives;

#[cfg(feature = "std")]
use version::NativeVersion;
use version::RuntimeVersion;

// A few exports that help ease life for downstream crates.
#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;
pub use timestamp::Call as TimestampCall;
pub use balances::Call as BalancesCall;
pub use sp_runtime::{Perbill, Permill};
pub use support::{
	StorageValue, construct_runtime, parameter_types,
	traits::Randomness,
	weights::Weight,
};

/// An index to a block.
pub type BlockNumber = u32;

/// Alias to 512-bit hash when used in the context of a transaction signature on the chain.
pub type Signature = MultiSignature;

/// Some way of identifying an account on the chain. We intentionally make it equivalent
/// to the public key of our transaction signing scheme.
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

/// The type for looking up accounts. We don't expect more than 4 billion of them, but you
/// never know...
pub type AccountIndex = u32;

/// Balance of an account.
pub type Balance = u128;

/// Index of a transaction in the chain.
pub type Index = u32;

/// A hash of some data used by the chain.
pub type Hash = primitives::H256;

/// Digest item type.
pub type DigestItem = generic::DigestItem<Hash>;

/// Opaque types. These are used by the CLI to instantiate machinery that don't need to know
/// the specifics of the runtime. They can then be made to be agnostic over specific formats
/// of data like extrinsics, allowing for them to continue syncing the network through upgrades
/// to even the core datastructures.
pub mod opaque {
    use super::*;

    pub use sp_runtime::OpaqueExtrinsic as UncheckedExtrinsic;

    /// Opaque block header type.
    pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
    /// Opaque block type.
    pub type Block = generic::Block<Header, UncheckedExtrinsic>;
    /// Opaque block identifier type.
    pub type BlockId = generic::BlockId<Block>;
//    pub type SessionHandlers = (Grandpa, Babe); // TODO delete this if it isn't needed
    impl_opaque_keys! {
        pub struct SessionKeys {
            pub grandpa: Grandpa, //TODO is this order correct? I changed stuff in chainspec.
            pub babe: Babe,
        }
    }
}

/// This runtime version.
pub const VERSION: RuntimeVersion = RuntimeVersion {
    spec_name: create_runtime_str!("super-runtime"),
    impl_name: create_runtime_str!("super-runtime"),
    authoring_version: 1,
    spec_version: 1,
    impl_version: 1,
    apis: RUNTIME_API_VERSIONS,
};

pub const MILLISECS_PER_BLOCK: u64 = 6000;

pub const SLOT_DURATION: u64 = MILLISECS_PER_BLOCK;

// These time units are defined in number of blocks.
pub const MINUTES: BlockNumber = 60_000 / (MILLISECS_PER_BLOCK as BlockNumber);
pub const HOURS: BlockNumber = MINUTES * 60;
pub const DAYS: BlockNumber = HOURS * 24;

// Some BABE-specific stuff
// 1 in 4 blocks (on average, not counting collisions) will be primary babe blocks.
pub const PRIMARY_PROBABILITY: (u64, u64) = (1, 4);
pub const EPOCH_DURATION_IN_BLOCKS: u32 = 10 * MINUTES;

/// The version infromation used to identify this runtime when compiled natively.
#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
    NativeVersion {
        runtime_version: VERSION,
        can_author_with: Default::default(),
    }
}

parameter_types! {
    pub const BlockHashCount: BlockNumber = 250;
    pub const MaximumBlockWeight: Weight = 1_000_000;
    pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
    pub const MaximumBlockLength: u32 = 5 * 1024 * 1024;
    pub const Version: RuntimeVersion = VERSION;
}

impl system::Trait for Runtime {
    /// The identifier used to distinguish between accounts.
    type AccountId = AccountId;
    /// The aggregated dispatch type that is available for extrinsics.
    type Call = Call;
    /// The lookup mechanism to get account ID from whatever is passed in dispatchers.
    type Lookup = Indices;
    /// The index type for storing how many extrinsics an account has signed.
    type Index = Index;
    /// The index type for blocks.
    type BlockNumber = BlockNumber;
    /// The type for hashing blocks and tries.
    type Hash = Hash;
    /// The hashing algorithm used.
    type Hashing = BlakeTwo256;
    /// The header type.
    type Header = generic::Header<BlockNumber, BlakeTwo256>;
    /// The ubiquitous event type.
    type Event = Event;
    /// The ubiquitous origin type.
    type Origin = Origin;
    /// Maximum number of block number to block hash mappings to keep (oldest pruned first).
    type BlockHashCount = BlockHashCount;
    /// Maximum weight of each block. With a default weight system of 1byte == 1weight, 4mb is ok.
    type MaximumBlockWeight = MaximumBlockWeight;
    /// Maximum size of all encoded transactions (in bytes) that are allowed in one block.
    type MaximumBlockLength = MaximumBlockLength;
    /// Portion of the block weight that is available to all normal transactions.
    type AvailableBlockRatio = AvailableBlockRatio;
    /// Version of the runtime.
	type Version = Version;
	/// Converts a module to the index of the module in `construct_runtime!`.
	///
	/// This type is being generated by `construct_runtime!`.
	type ModuleToIndex = ModuleToIndex;
}

parameter_types! {
    pub const EpochDuration: u64 = EPOCH_DURATION_IN_BLOCKS as u64;
    pub const ExpectedBlockTime: u64 = MILLISECS_PER_BLOCK;
}

impl babe::Trait for Runtime {
    type EpochDuration = EpochDuration;
    type ExpectedBlockTime = ExpectedBlockTime;
    type EpochChangeTrigger = SameAuthoritiesForever;
}

impl grandpa::Trait for Runtime {
    type Event = Event;
}

impl indices::Trait for Runtime {
    /// The type for recording indexing into the account enumeration. If this ever overflows, there
    /// will be problems!
    type AccountIndex = AccountIndex;
    /// Use the standard means of resolving an index hint from an id.
    type ResolveHint = indices::SimpleResolveHint<Self::AccountId, Self::AccountIndex>;
    /// Determine whether an account is dead.
    type IsDeadAccount = Balances;
    /// The ubiquitous event type.
    type Event = Event;
}

parameter_types! {
    pub const MinimumPeriod: u64 = SLOT_DURATION / 2;
}

impl timestamp::Trait for Runtime {
    /// A timestamp: milliseconds since the unix epoch.
    type Moment = u64;
    type OnTimestampSet = Babe;
    type MinimumPeriod = MinimumPeriod;
}

parameter_types! {
    pub const ExistentialDeposit: u128 = 500;
    pub const TransferFee: u128 = 0;
    pub const CreationFee: u128 = 0;
}

impl balances::Trait for Runtime {
    /// The type for recording an account's balance.
    type Balance = Balance;
    /// What to do if an account's free balance gets zeroed.
    type OnFreeBalanceZero = ();
    /// What to do if a new account is created.
    type OnNewAccount = Indices;
    /// The ubiquitous event type.
    type Event = Event;
    type DustRemoval = ();
    type TransferPayment = ();
    type ExistentialDeposit = ExistentialDeposit;
    type TransferFee = TransferFee;
    type CreationFee = CreationFee;
}

parameter_types! {
    pub const TransactionBaseFee: u128 = 0;
    pub const TransactionByteFee: u128 = 1;
}

impl transaction_payment::Trait for Runtime {
    type Currency = balances::Module<Runtime>;
    type OnTransactionPayment = ();
    type TransactionBaseFee = TransactionBaseFee;
    type TransactionByteFee = TransactionByteFee;
    type WeightToFee = ConvertInto;
    type FeeMultiplierUpdate = ();
}

impl sudo::Trait for Runtime {
    type Event = Event;
    type Proposal = Call;
}

// ---------------------- Recipe Pallet Configurations ----------------------
impl adding_machine::Trait for Runtime {}

impl basic_token::Trait for Runtime {
    type Event = Event;
}

impl charity::Trait for Runtime {
    type Event = Event;
    type Currency = Balances;
}

parameter_types! {
    pub const MaxAddend: u32 = 1738;
    pub const ClearFrequency: u32 = 10;
}

impl constant_config::Trait for Runtime {
    type Event = Event;
    type MaxAddend = MaxAddend;
    type ClearFrequency = ClearFrequency;
}

impl check_membership::Trait for Runtime {
    type Event = Event;
}

// The following two configuration traits are for two different instances of the deafult-instance
// pallet. Notice that only the second instance has to explicitly specify an instance.
impl default_instance::Trait for Runtime {
    type Event = Event;
}

impl default_instance::Trait<default_instance::Instance2> for Runtime {
    type Event = Event;
}

impl double_map::Trait for Runtime {
    type Event = Event;
}

parameter_types! {
    pub const ExecutionFrequency: u32 = 10;
    pub const SignalQuota: u32 = 1000;
    pub const TaskLimit: u32 = 10;
}

impl execution_schedule::Trait for Runtime {
    type Event = Event;
    type ExecutionFrequency = ExecutionFrequency;
    type SignalQuota = SignalQuota;
    type TaskLimit = TaskLimit;
}

impl generic_event::Trait for Runtime {
    type Event = Event;
}

impl hello_substrate::Trait for Runtime {}

// The following two configuration traits are for two different instances of the last-caller pallet
impl last_caller::Trait<last_caller::Instance1> for Runtime {
    type Event = Event;
}

impl last_caller::Trait<last_caller::Instance2> for Runtime {
    type Event = Event;
}

impl linked_map::Trait for Runtime {
    type Event = Event;
}

impl simple_event::Trait for Runtime {
    type Event = Event;
}

impl simple_map::Trait for Runtime {
    type Event = Event;
}

impl single_value::Trait for Runtime {}

impl storage_cache::Trait for Runtime {
    type Event = Event;
}

impl struct_storage::Trait for Runtime {
    type Event = Event;
}

impl vec_set::Trait for Runtime {
    type Event = Event;
}

construct_runtime!(
	pub enum Runtime where
		Block = Block,
		NodeBlock = opaque::Block,
		UncheckedExtrinsic = UncheckedExtrinsic
	{
		System: system::{Module, Call, Storage, Config, Event},
		Timestamp: timestamp::{Module, Call, Storage, Inherent},
		Babe: babe::{Module, Call, Storage, Config, Inherent(Timestamp)},
		Grandpa: grandpa::{Module, Call, Storage, Config, Event},
		Indices: indices,
		Balances: balances,
		RandomnessCollectiveFlip: randomness_collective_flip::{Module, Call, Storage},
		Sudo: sudo,
		TransactionPayment: transaction_payment::{Module, Storage},
		// The Recipe Pallets
		AddingMachine: adding_machine::{Module, Call, Storage},
		BasicToken: basic_token::{Module, Call, Storage, Event<T>},
		Charity: charity::{Module, Call, Storage, Event<T>},
		CheckMembership: check_membership::{Module, Call, Storage, Event<T>},
		ConstantConfig: constant_config::{Module, Call, Storage, Event},
		DefaultInstance1: default_instance::{Module, Call, Storage, Event<T>},
		DefaultInstance2: default_instance::<Instance2>::{Module, Call, Storage, Event<T>},
		DoubleMap: double_map::{Module, Call, Storage, Event<T>},
		ExecutionSchedule: execution_schedule::{Module, Call, Storage, Event<T>},
		HelloSubstrate: hello_substrate::{Module, Call},
		GenericEvent: generic_event::{Module, Call, Event<T>},
		LastCaller1: last_caller::<Instance1>::{Module, Call, Storage, Event<T>},
		LastCaller2: last_caller::<Instance2>::{Module, Call, Storage, Event<T>},
		LinkedMap: linked_map::{Module, Call, Storage, Event<T>},
		SimpleEvent: simple_event::{Module, Call, Event},
		SimpleMap: simple_map::{Module, Call, Storage, Event<T>},
		SingleValue: single_value::{Module, Call, Storage},
		StorageCache: storage_cache::{Module, Call, Storage, Event<T>},
		StructStorage: struct_storage::{Module, Call, Storage, Event<T>},
		VecSet: vec_set::{Module, Call, Storage, Event<T>},
	}
);

/// The address format for describing accounts.
pub type Address = <Indices as StaticLookup>::Source;
/// Block header type as expected by this runtime.
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
/// Block type as expected by this runtime.
pub type Block = generic::Block<Header, UncheckedExtrinsic>;
/// A Block signed with a Justification
pub type SignedBlock = generic::SignedBlock<Block>;
/// BlockId type as expected by this runtime.
pub type BlockId = generic::BlockId<Block>;
/// The SignedExtension to the basic transaction logic.
pub type SignedExtra = (
    system::CheckVersion<Runtime>,
    system::CheckGenesis<Runtime>,
    system::CheckEra<Runtime>,
    system::CheckNonce<Runtime>,
    system::CheckWeight<Runtime>,
    transaction_payment::ChargeTransactionPayment<Runtime>,
);
/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic = generic::UncheckedExtrinsic<Address, Call, Signature, SignedExtra>;
/// Extrinsic type that has already been checked.
pub type CheckedExtrinsic = generic::CheckedExtrinsic<AccountId, Call, SignedExtra>;
/// Executive: handles dispatch to the various pallets.
pub type Executive = executive::Executive<Runtime, Block, system::ChainContext<Runtime>, Runtime, AllModules>;

impl_runtime_apis! {
    impl sp_api::Core<Block> for Runtime {
        fn version() -> RuntimeVersion {
            VERSION
        }

        fn execute_block(block: Block) {
            Executive::execute_block(block)
        }

        fn initialize_block(header: &<Block as BlockT>::Header) {
            Executive::initialize_block(header)
        }
    }

    impl sp_api::Metadata<Block> for Runtime {
        fn metadata() -> OpaqueMetadata {
            Runtime::metadata().into()
        }
    }

    impl block_builder_api::BlockBuilder<Block> for Runtime {
        fn apply_extrinsic(extrinsic: <Block as BlockT>::Extrinsic) -> ApplyExtrinsicResult {
            Executive::apply_extrinsic(extrinsic)
        }

        fn finalize_block() -> <Block as BlockT>::Header {
            Executive::finalize_block()
        }

        fn inherent_extrinsics(data: inherents::InherentData) -> Vec<<Block as BlockT>::Extrinsic> {
            data.create_extrinsics()
        }

        fn check_inherents(
            block: Block,
            data: inherents::InherentData
        ) -> inherents::CheckInherentsResult {
            data.check_extrinsics(&block)
        }

        fn random_seed() -> <Block as BlockT>::Hash {
            RandomnessCollectiveFlip::random_seed()
        }
    }

    impl sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block> for Runtime {
        fn validate_transaction(tx: <Block as BlockT>::Extrinsic) -> TransactionValidity {
            Executive::validate_transaction(tx)
        }
    }

    impl offchain_primitives::OffchainWorkerApi<Block> for Runtime {
        fn offchain_worker(number: NumberFor<Block>) {
            Executive::offchain_worker(number)
        }
    }

    impl fg_primitives::GrandpaApi<Block> for Runtime {
        fn grandpa_authorities() -> GrandpaAuthorityList {
            Grandpa::grandpa_authorities()
        }
    }

    impl babe_primitives::BabeApi<Block> for Runtime {
        fn configuration() -> babe_primitives::BabeConfiguration {
            // The choice of `c` parameter (where `1 - c` represents the
            // probability of a slot being empty), is done in accordance to the
            // slot duration and expected target block time, for safely
            // resisting network delays of maximum two seconds.
            // <https://research.web3.foundation/en/latest/polkadot/BABE/Babe/#6-practical-results>
            babe_primitives::BabeConfiguration {
                slot_duration: Babe::slot_duration(),
                epoch_length: EpochDuration::get(),
                c: PRIMARY_PROBABILITY,
                genesis_authorities: Babe::authorities(),
                randomness: Babe::randomness(),
                secondary_slots: true,
            }
        }
    }

    impl substrate_session::SessionKeys<Block> for Runtime {
        fn generate_session_keys(seed: Option<Vec<u8>>) -> Vec<u8> {
            opaque::SessionKeys::generate(seed)
        }
    }
}
