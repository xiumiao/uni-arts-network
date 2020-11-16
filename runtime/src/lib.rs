#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit="256"]

// Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

pub mod constants;

/// Weights for pallets used in the runtime.
mod weights;

use sp_std::prelude::*;
use sp_core::{crypto::KeyTypeId, OpaqueMetadata};
use sp_runtime::{
	ApplyExtrinsicResult, generic, create_runtime_str, impl_opaque_keys,
	transaction_validity::{TransactionValidity, TransactionSource}
};
use sp_runtime::traits::{
	BlakeTwo256, Block as BlockT, IdentityLookup, NumberFor, Saturating, ConvertInto, AccountIdConversion,
	Convert, OpaqueKeys, SaturatedConversion, Bounded
};
use frame_system::{EnsureOneOf, EnsureRoot};
use sp_api::impl_runtime_apis;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_core::{u32_trait::{_1, _2}, };
use pallet_grandpa::{AuthorityId as GrandpaId, AuthorityList as GrandpaAuthorityList};
use pallet_grandpa::fg_primitives;
use sp_version::RuntimeVersion;
#[cfg(feature = "std")]
use sp_version::NativeVersion;

// Uni-Arts
use constants::{currency::*};
type Uart = Balances;

// A few exports that help ease life for downstream crates.
#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;
pub use sp_runtime::{Permill, Perbill, Percent, ModuleId};
pub use pallet_timestamp::Call as TimestampCall;
pub use pallet_balances::Call as BalancesCall;
pub use frame_support::{
	construct_runtime, parameter_types, StorageValue,
	traits::{KeyOwnerProofSystem, Randomness, StorageMapShim, Currency, Contains, ContainsLengthBound},
	weights::{
		Weight, IdentityFee,
		constants::{BlockExecutionWeight, ExtrinsicBaseWeight, RocksDbWeight, WEIGHT_PER_SECOND},
	},
};

pub use primitives::{
	BlockNumber, Signature, AccountId, AccountIndex, Balance, Index, Hash, DigestItem,
	TokenSymbol, CurrencyId,
};

/// Import pallets.
pub use pallet_certificate;
pub use pallet_assets;
pub use pallet_nft;
pub use pallet_nicks;
pub use pallet_rewards;
pub use pallet_staking;
pub use pallet_validator_set;
pub use pallet_token;
pub use pallet_trade;


/// Opaque types. These are used by the CLI to instantiate machinery that don't need to know
/// the specifics of the runtime. They can then be made to be agnostic over specific formats
/// of data like extrinsics, allowing for them to continue syncing the network through upgrades
/// to even the core data structures.
pub mod opaque {
	use super::*;

	pub use sp_runtime::OpaqueExtrinsic as UncheckedExtrinsic;

	/// Opaque block header type.
	pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
	/// Opaque block type.
	pub type Block = generic::Block<Header, UncheckedExtrinsic>;
	/// Opaque block identifier type.
	pub type BlockId = generic::BlockId<Block>;

	impl_opaque_keys! {
		pub struct SessionKeys {
			pub aura: Aura,
			pub grandpa: Grandpa,
		}
	}
}

pub const VERSION: RuntimeVersion = RuntimeVersion {
	spec_name: create_runtime_str!("uart"),
	impl_name: create_runtime_str!("uart"),
	authoring_version: 1,
	spec_version: 1,
	impl_version: 1,
	apis: RUNTIME_API_VERSIONS,
	transaction_version: 1,
};

pub const MILLISECS_PER_BLOCK: u64 = 6000;

pub const SLOT_DURATION: u64 = MILLISECS_PER_BLOCK;

// Time is measured by number of blocks.
pub const MINUTES: BlockNumber = 60_000 / (MILLISECS_PER_BLOCK as BlockNumber);
pub const HOURS: BlockNumber = MINUTES * 60;
pub const DAYS: BlockNumber = HOURS * 24;

/// The version information used to identify this runtime when compiled natively.
#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
	NativeVersion {
		runtime_version: VERSION,
		can_author_with: Default::default(),
	}
}

// Module accounts of runtime
parameter_types! {
	pub const UniArtsTreasuryModuleId: ModuleId = ModuleId(*b"art/trsy");
	pub const StakingModuleId: ModuleId = ModuleId(*b"staking_");
	pub const UniArtsNftModuleId: ModuleId = ModuleId(*b"art/nftb");
	pub ZeroAccountId: AccountId = AccountId::from([0u8; 32]);
}

pub fn get_all_module_accounts() -> Vec<AccountId> {
	vec![
		UniArtsTreasuryModuleId::get().into_account(),
		ZeroAccountId::get(),
	]
}

parameter_types! {
	pub const BlockHashCount: BlockNumber = 2400;
	/// We allow for 2 seconds of compute with a 6 second average block time.
	pub const MaximumBlockWeight: Weight = 2 * WEIGHT_PER_SECOND;
	pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
	/// Assume 10% of weight for average on_initialize calls.
	pub MaximumExtrinsicWeight: Weight = AvailableBlockRatio::get()
		.saturating_sub(Perbill::from_percent(10)) * MaximumBlockWeight::get();
	pub const MaximumBlockLength: u32 = 5 * 1024 * 1024;
	pub const Version: RuntimeVersion = VERSION;
}

// Configure FRAME pallets to include in runtime.

impl frame_system::Trait for Runtime {
	/// The basic call filter to use in dispatchable.
	type BaseCallFilter = ();
	/// The identifier used to distinguish between accounts.
	type AccountId = AccountId;
	/// The aggregated dispatch type that is available for extrinsics.
	type Call = Call;
	/// The lookup mechanism to get account ID from whatever is passed in dispatchers.
	type Lookup = IdentityLookup<AccountId>;
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
	/// Maximum weight of each block.
	type MaximumBlockWeight = MaximumBlockWeight;
	/// The weight of database operations that the runtime can invoke.
	type DbWeight = RocksDbWeight;
	/// The weight of the overhead invoked on the block import process, independent of the
	/// extrinsics included in that block.
	type BlockExecutionWeight = BlockExecutionWeight;
	/// The base weight of any extrinsic processed by the runtime, independent of the
	/// logic of that extrinsic. (Signature verification, nonce increment, fee, etc...)
	type ExtrinsicBaseWeight = ExtrinsicBaseWeight;
	/// The maximum weight that a single extrinsic of `Normal` dispatch class can have,
	/// idependent of the logic of that extrinsics. (Roughly max block weight - average on
	/// initialize cost).
	type MaximumExtrinsicWeight = MaximumExtrinsicWeight;
	/// Maximum size of all encoded transactions (in bytes) that are allowed in one block.
	type MaximumBlockLength = MaximumBlockLength;
	/// Portion of the block weight that is available to all normal transactions.
	type AvailableBlockRatio = AvailableBlockRatio;
	/// Version of the runtime.
	type Version = Version;

	type PalletInfo = PalletInfo;
	/// What to do if a new account is created.
	type OnNewAccount = ();
	/// What to do if an account is fully reaped from the system.
	type OnKilledAccount = ();
	/// The data to be stored in an account.
	type AccountData = pallet_balances::AccountData<Balance>;
	/// Weight information for the extrinsics of this pallet.
	type SystemWeightInfo = ();
}

parameter_types! {
	pub const DisabledValidatorsThreshold: Perbill = Perbill::from_percent(17);
	pub const Offset: BlockNumber = 0;
	pub const Period: BlockNumber = 5;
	pub const ValidatorMortgageLimit: Balance = 100 * UART;
}

impl pallet_validator_set::Trait for Runtime {
	type Event = Event;
	type Currency = Balances;
	type ValidatorMortgageLimit = ValidatorMortgageLimit;
}

pub struct ValidatorIdOf;
impl<T> Convert<T, Option<T>> for ValidatorIdOf {
	fn convert(a: T) -> Option<T> { 
		Some(a)
	}
}

impl pallet_session::Trait for Runtime {
	type Event = Event;
	type ValidatorId = AccountId;
	type ValidatorIdOf = ValidatorIdOf;
	type ShouldEndSession = ValidatorSet;
	type NextSessionRotation = ValidatorSet;
	type SessionManager = ValidatorSet;
	type SessionHandler = <opaque::SessionKeys as OpaqueKeys>::KeyTypeIdProviders;
	type Keys = opaque::SessionKeys;
	type DisabledValidatorsThreshold = DisabledValidatorsThreshold;
	type WeightInfo = ();
}

impl pallet_aura::Trait for Runtime {
	type AuthorityId = AuraId;
}

impl pallet_grandpa::Trait for Runtime {
	type Event = Event;
	type Call = Call;

	type KeyOwnerProofSystem = ();

	type KeyOwnerProof =
		<Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(KeyTypeId, GrandpaId)>>::Proof;

	type KeyOwnerIdentification = <Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(
		KeyTypeId,
		GrandpaId,
	)>>::IdentificationTuple;

	type HandleEquivocation = ();

	type WeightInfo = ();
}

parameter_types! {
	pub const MiningRewardPerBlock: Balance = 1 * UART;
	pub const StakingRewardPerBlock: Balance = 1 * UART;
	pub const AmpFactor: Balance = 1e12 as Balance;
}

pub struct AccoundIdOf;
impl<T> Convert<T, Option<T>> for AccoundIdOf {
	fn convert(a: T) -> Option<T> { 
		Some(a)
	}
}

impl pallet_rewards::Trait for Runtime {
	type AccoundIdOf = AccoundIdOf;
	type Balance = Balance;
	type Currency = Uart;
	type RewardPerBlock = MiningRewardPerBlock;
	type Event = Event;
}

pub struct ConvertNumberToBalance;
impl<BlockNumber, Balance: Bounded + core::convert::From<BlockNumber>> Convert<BlockNumber, Balance> for ConvertNumberToBalance {
	fn convert(a: BlockNumber) -> Balance {
		Balance::saturated_from::<BlockNumber>(a)
	}
}


impl pallet_staking::Trait for Runtime {
	type ModuleId = StakingModuleId;
	type Event = Event;
	type Currency = Uart;
	type RewardPerBlock = StakingRewardPerBlock;
	type Id = u32;
	type AmpFactor = AmpFactor;
	type ConvertNumberToBalance = ConvertNumberToBalance;
}

parameter_types! {
	pub const MinVestedTransfer: Balance = 100 * DOLLARS;
}

impl pallet_vesting::Trait for Runtime {
	type Event = Event;
	type Currency = Uart;
	type BlockNumberToBalance = ConvertInto;
	type MinVestedTransfer = MinVestedTransfer;
	type WeightInfo = ();
}

parameter_types! {
	pub const MinimumPeriod: u64 = SLOT_DURATION / 2;
}

impl pallet_timestamp::Trait for Runtime {
	/// A timestamp: milliseconds since the unix epoch.
	type Moment = u64;
	type OnTimestampSet = Aura;
	type MinimumPeriod = MinimumPeriod;
	type WeightInfo = ();
}

parameter_types! {
	pub const ExistentialDeposit: u128 = 500;
	// For weight estimation, we assume that the most locks on an individual account will be 50.
	// This number may need to be adjusted in the future if this assumption no longer holds true.
	pub const MaxLocks: u32 = 50;
}

// type UartInstance = pallet_balances::Instance0;
type UinkInstance = pallet_balances::Instance1;

impl pallet_balances::Trait for Runtime {
	/// The type for recording an account's balance.
	type Balance = Balance;
	/// The ubiquitous event type.
	type Event = Event;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
	type MaxLocks = MaxLocks;
}

// impl pallet_balances::Trait<UartInstance> for Runtime {
// 	/// The type for recording an account's balance.
// 	type Balance = Balance;
// 	/// The ubiquitous event type.
// 	type Event = Event;
// 	type DustRemoval = ();
// 	type ExistentialDeposit = ExistentialDeposit;
// 	type AccountStore = System;
// 	type WeightInfo = ();
// }

type UinkAccountStore = StorageMapShim<
		pallet_balances::Account<Runtime, UinkInstance>,
		frame_system::CallOnCreatedAccount<Runtime>,
		frame_system::CallKillAccount<Runtime>,
		AccountId,
		pallet_balances::AccountData<Balance>
	>;

impl pallet_balances::Trait<UinkInstance> for Runtime {
	/// The type for recording an account's balance.
	type Balance = Balance;
	/// The ubiquitous event type.
	type Event = Event;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = UinkAccountStore;
	type WeightInfo = ();
	type MaxLocks = MaxLocks;
}

parameter_types! {
	pub const TransactionByteFee: Balance = 1;
}

impl pallet_transaction_payment::Trait for Runtime {
	type Currency = Uart;
	type OnTransactionPayment = ();
	type TransactionByteFee = TransactionByteFee;
	type WeightToFee = IdentityFee<Balance>;
	type FeeMultiplierUpdate = ();
}

parameter_types! {
    // Choose a fee that incentivizes desireable behavior.
    pub const NickReservationFee: u128 = 100;
    pub const MinNickLength: usize = 6;
    // Maximum bounds on storage are important to secure your chain.
    pub const MaxNickLength: usize = 32;
}

impl pallet_nicks::Trait for Runtime {
	/// The Balances pallet implements the ReservableCurrency trait.
	type Currency = Uart;
	/// Use the NickReservationFee from the parameter_types block.
	type ReservationFee = NickReservationFee;
	/// No action is taken when deposits are forfeited.
	type Slashed = ();
	/// Configure the FRAME System Root origin as the Nick pallet admin.
	type ForceOrigin = frame_system::EnsureRoot<AccountId>;
	/// Use the MinNickLength from the parameter_types block.
	type MinLength = MinNickLength;
	/// Use the MaxNickLength from the parameter_types block.
	type MaxLength = MaxNickLength;
	/// The ubiquitous event type.
	type Event = Event;
}

impl pallet_sudo::Trait for Runtime {
	type Event = Event;
	type Call = Call;
}

impl pallet_certificate::Trait for Runtime {
	type Event = Event;
	type WorkId = u32;
}

impl pallet_assets::Trait for Runtime {
	type Event = Event;
	type Balance = Balance;
	type AssetId = u32;
}

impl pallet_token::Trait for Runtime {
	type Event = Event;
}

parameter_types! {
	pub const PriceFactor: u128 = 100_000_000;
    pub const BlocksPerDay: u32 = 6 * 60 * 24;
    pub const OpenedOrdersArrayCap: u8 = 20;
    pub const ClosedOrdersArrayCap: u8 = 100;
}

impl pallet_trade::Trait for Runtime {
	type Event = Event;
	type Price = u128;
	type PriceFactor = PriceFactor;
	type BlocksPerDay = BlocksPerDay;
	type OpenedOrdersArrayCap = OpenedOrdersArrayCap;
	type ClosedOrdersArrayCap = ClosedOrdersArrayCap;
}

impl pallet_names::Trait for Runtime {
	type Name = Vec<u8>;
	type Value = Vec<u8>;
	type Currency = Uart;
	type Event = Event;

	fn get_name_fee(op: &pallet_names::Operation<Self>) -> Option<Balance> {
		/* Single-letter names are not allowed (nor the empty name).  Everything
           else is fine.  */
		if op.name.len() < 2 {
			return None
		}

		Some(match op.operation {
			pallet_names::OperationType::Registration => 1000,
			pallet_names::OperationType::Update => 100,
		})
	}

	fn get_expiration(op: &pallet_names::Operation<Self>) -> Option<BlockNumber> {
		/* Short names (up to three characters) will expire after 10 blocks.
           Longer names will stick around forever.  */
		if op.name.len() <= 3 {
			Some(10)
		} else {
			None
		}
	}

	fn deposit_fee(_b: <Self::Currency as Currency<AccountId>>::NegativeImbalance) {
		/* Just burn the name fee by dropping the imbalance.  */
	}

}

/// Used for the module nft in `./nft.rs`
impl pallet_nft::Trait for Runtime {
	type ModuleId = UniArtsNftModuleId;
	type Currency = Uart;
	type Event = Event;
}

// Uni-Art Treasury
parameter_types! {
	pub const GeneralCouncilMotionDuration: BlockNumber = 0;
	pub const GeneralCouncilMaxProposals: u32 = 100;
	pub const CouncilMaxProposals: u32 = 100;
	pub const CouncilMaxMembers: u32 = 100;
}

type GeneralCouncilInstance = pallet_collective::Instance1;
impl pallet_collective::Trait<GeneralCouncilInstance> for Runtime {
	type Origin = Origin;
	type Proposal = Call;
	type Event = Event;
	type MotionDuration = GeneralCouncilMotionDuration;
	type MaxProposals = GeneralCouncilMaxProposals;
	type WeightInfo = ();
	type MaxMembers = CouncilMaxMembers;
	type DefaultVote = pallet_collective::PrimeDefaultVote;
}

pub struct GeneralCouncilProvider;
impl Contains<AccountId> for GeneralCouncilProvider {
	fn contains(who: &AccountId) -> bool {
		GeneralCouncil::is_member(who)
	}

	fn sorted_members() -> Vec<AccountId> {
		GeneralCouncil::members()
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn add(_: &AccountId) {
		todo!()
	}
}

impl ContainsLengthBound for GeneralCouncilProvider {
	fn max_len() -> usize {
		100
	}
	fn min_len() -> usize {
		0
	}
}

// Uni-Art Treasury
parameter_types! {
	pub const ProposalBond: Permill = Permill::from_percent(5);
	pub const ProposalBondMinimum: Balance = 1 * UART;
	pub const SpendPeriod: BlockNumber = 1 * DAYS;
	pub const Burn: Permill = Permill::from_percent(0);
	pub const TipCountdown: BlockNumber = 1 * DAYS;
	pub const TipFindersFee: Percent = Percent::from_percent(10);
	pub const TipReportDepositBase: Balance = 1 * UART;
	pub const SevenDays: BlockNumber = 7 * DAYS;
	pub const ZeroDay: BlockNumber = 0;
	pub const OneDay: BlockNumber = DAYS;
	pub const DataDepositPerByte: Balance = 1 * MILLI;
	pub const BountyDepositBase: Balance = 1 * UART;
	pub const BountyDepositPayoutDelay: BlockNumber = 4 * DAYS;
	pub const BountyUpdatePeriod: BlockNumber = 90 * DAYS;
	pub const BountyCuratorDeposit: Permill = Permill::from_percent(50);
	pub const BountyValueMinimum: Balance = 2 * UART;
	pub const MaximumReasonLength: u32 = 16384;
}

impl pallet_treasury::Trait for Runtime {
	type ModuleId = UniArtsTreasuryModuleId;
	type Currency = Uart;
	type ApproveOrigin = frame_system::EnsureRoot<AccountId>;
	type RejectOrigin = frame_system::EnsureRoot<AccountId>;
	type Tippers = GeneralCouncilProvider;
	type TipCountdown = TipCountdown;
	type TipFindersFee = TipFindersFee;
	type TipReportDepositBase = TipReportDepositBase;
	type Event = Event;
	type ProposalBond = ProposalBond;
	type ProposalBondMinimum = ProposalBondMinimum;
	type SpendPeriod = SpendPeriod;
	type Burn = Burn;
	type BurnDestination = ();
	type WeightInfo = ();
	type DataDepositPerByte = DataDepositPerByte;
	type OnSlash = UniArtsTreasury;
	type BountyDepositBase = BountyDepositBase;
	type BountyDepositPayoutDelay = BountyDepositPayoutDelay;
	type BountyUpdatePeriod = BountyUpdatePeriod;
	type BountyCuratorDeposit = BountyCuratorDeposit;
	type BountyValueMinimum = BountyValueMinimum;
	type MaximumReasonLength = MaximumReasonLength;
}

type EnsureRootOrMoreThanHalfCouncil = EnsureOneOf<
	AccountId,
	EnsureRoot<AccountId>,
	pallet_collective::EnsureProportionMoreThan<_1, _2, AccountId, GeneralCouncilInstance>,
>;

parameter_types! {
	pub const BasicDeposit: Balance = 10 * UART;            // 258 bytes on-chain
	pub const FieldDeposit: Balance = 250 * MICRO;          // 66 bytes on-chain
	pub const SubAccountDeposit: Balance = 2 * UART;        // 53 bytes on-chain
	pub const MaxSubAccounts: u32 = 100;
	pub const MaxAdditionalFields: u32 = 100;
	pub const MaxRegistrars: u32 = 20;
}
impl pallet_identity::Trait for Runtime {
	type Event = Event;
	type Currency = Uart;
	type BasicDeposit = BasicDeposit;
	type FieldDeposit = FieldDeposit;
	type SubAccountDeposit = SubAccountDeposit;
	type MaxSubAccounts = MaxSubAccounts;
	type MaxAdditionalFields = MaxAdditionalFields;
	type MaxRegistrars = MaxRegistrars;
	type Slashed = UniArtsTreasury;
	type ForceOrigin = EnsureRootOrMoreThanHalfCouncil;
	type RegistrarOrigin = EnsureRootOrMoreThanHalfCouncil;
	type WeightInfo = ();
}

parameter_types! {
	pub const MaxScheduledPerBlock: u32 = 50;
}
impl pallet_scheduler::Trait for Runtime {
	type Event = Event;
	type Origin = Origin;
	type PalletsOrigin = OriginCaller;
	type Call = Call;
	type MaximumWeight = MaximumBlockWeight;
	type ScheduleOrigin = EnsureRoot<AccountId>;
	type MaxScheduledPerBlock = MaxScheduledPerBlock;
	type WeightInfo = weights::pallet_scheduler::WeightInfo<Runtime>;
}

// Create the runtime by composing the FRAME pallets that were previously configured.
construct_runtime!(
	pub enum Runtime where
		Block = Block,
		NodeBlock = opaque::Block,
		UncheckedExtrinsic = UncheckedExtrinsic
	{
		System: frame_system::{Module, Call, Config, Storage, Event<T>},
		RandomnessCollectiveFlip: pallet_randomness_collective_flip::{Module, Call, Storage},
		Timestamp: pallet_timestamp::{Module, Call, Storage, Inherent},

		Session: pallet_session::{Module, Call, Storage, Event, Config<T>},
		ValidatorSet: pallet_validator_set::{Module, Call, Storage, Event<T>, Config<T>},
		Aura: pallet_aura::{Module, Config<T>, Inherent},
		Grandpa: pallet_grandpa::{Module, Call, Storage, Config, Event},
		Rewards: pallet_rewards::{Module, Call, Storage, Event<T>},
		Staking: pallet_staking::{Module, Call, Storage, Event<T>},
		Vesting: pallet_vesting::{Module, Call, Storage, Event<T>, Config<T>},

		Nicks: pallet_nicks::{Module, Call, Storage, Event<T>},
		Balances: pallet_balances::{Module, Call, Storage, Config<T>, Event<T>},
		// Uart: pallet_balances::<Instance0>::{Module, Call, Storage, Config<T>, Event<T>},
		Uink: pallet_balances::<Instance1>::{Module, Call, Storage, Config<T>, Event<T>},

		// Governance
		GeneralCouncil: pallet_collective::<Instance1>::{Module, Call, Storage, Origin<T>, Event<T>, Config<T>},
		UniArtsTreasury: pallet_treasury::{Module, Call, Storage, Config, Event<T>},
		Identity: pallet_identity::{Module, Call, Storage, Event<T>},

		// System scheduler.
		Scheduler: pallet_scheduler::{Module, Call, Storage, Event<T>},
		TransactionPayment: pallet_transaction_payment::{Module, Storage},
		Sudo: pallet_sudo::{Module, Call, Config<T>, Storage, Event<T>},

		Certificate: pallet_certificate::{Module, Call, Storage, Event<T>},
		Assets: pallet_assets::{Module, Call, Storage, Event<T>},
		Names: pallet_names::{Module, Call, Storage, Event<T>},
		Nft: pallet_nft::{Module, Call, Storage, Event<T>},
		Token: pallet_token::{Module, Call, Storage, Event<T>},
		Trade: pallet_trade::{Module, Call, Storage, Event<T>},
	}
);

/// The address format for describing accounts.
pub type Address = AccountId;
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
	frame_system::CheckSpecVersion<Runtime>,
	frame_system::CheckTxVersion<Runtime>,
	frame_system::CheckGenesis<Runtime>,
	frame_system::CheckEra<Runtime>,
	frame_system::CheckNonce<Runtime>,
	frame_system::CheckWeight<Runtime>,
	pallet_transaction_payment::ChargeTransactionPayment<Runtime>
);
/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic = generic::UncheckedExtrinsic<Address, Call, Signature, SignedExtra>;
/// Extrinsic type that has already been checked.
pub type CheckedExtrinsic = generic::CheckedExtrinsic<AccountId, Call, SignedExtra>;
/// Executive: handles dispatch to the various modules.
pub type Executive = frame_executive::Executive<
	Runtime,
	Block,
	frame_system::ChainContext<Runtime>,
	Runtime,
	AllModules,
>;

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

	impl sp_block_builder::BlockBuilder<Block> for Runtime {
		fn apply_extrinsic(extrinsic: <Block as BlockT>::Extrinsic) -> ApplyExtrinsicResult {
			Executive::apply_extrinsic(extrinsic)
		}

		fn finalize_block() -> <Block as BlockT>::Header {
			Executive::finalize_block()
		}

		fn inherent_extrinsics(data: sp_inherents::InherentData) -> Vec<<Block as BlockT>::Extrinsic> {
			data.create_extrinsics()
		}

		fn check_inherents(
			block: Block,
			data: sp_inherents::InherentData,
		) -> sp_inherents::CheckInherentsResult {
			data.check_extrinsics(&block)
		}

		fn random_seed() -> <Block as BlockT>::Hash {
			RandomnessCollectiveFlip::random_seed()
		}
	}

	impl sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block> for Runtime {
		fn validate_transaction(
			source: TransactionSource,
			tx: <Block as BlockT>::Extrinsic,
		) -> TransactionValidity {
			Executive::validate_transaction(source, tx)
		}
	}

	impl sp_offchain::OffchainWorkerApi<Block> for Runtime {
		fn offchain_worker(header: &<Block as BlockT>::Header) {
			Executive::offchain_worker(header)
		}
	}

	impl sp_consensus_aura::AuraApi<Block, AuraId> for Runtime {
		fn slot_duration() -> u64 {
			Aura::slot_duration()
		}

		fn authorities() -> Vec<AuraId> {
			Aura::authorities()
		}
	}

	impl sp_session::SessionKeys<Block> for Runtime {
		fn generate_session_keys(seed: Option<Vec<u8>>) -> Vec<u8> {
			opaque::SessionKeys::generate(seed)
		}

		fn decode_session_keys(
			encoded: Vec<u8>,
		) -> Option<Vec<(Vec<u8>, KeyTypeId)>> {
			opaque::SessionKeys::decode_into_raw_public_keys(&encoded)
		}
	}

	impl fg_primitives::GrandpaApi<Block> for Runtime {
		fn grandpa_authorities() -> GrandpaAuthorityList {
			Grandpa::grandpa_authorities()
		}

		fn submit_report_equivocation_unsigned_extrinsic(
			_equivocation_proof: fg_primitives::EquivocationProof<
				<Block as BlockT>::Hash,
				NumberFor<Block>,
			>,
			_key_owner_proof: fg_primitives::OpaqueKeyOwnershipProof,
		) -> Option<()> {
			None
		}

		fn generate_key_ownership_proof(
			_set_id: fg_primitives::SetId,
			_authority_id: GrandpaId,
		) -> Option<fg_primitives::OpaqueKeyOwnershipProof> {
			// NOTE: this is the only implementation possible since we've
			// defined our key owner proof type as a bottom type (i.e. a type
			// with no values).
			None
		}
	}
	
	impl frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Index> for Runtime {
		fn account_nonce(account: AccountId) -> Index {
			System::account_nonce(account)
		}
	}

	impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, Balance> for Runtime {
		fn query_info(
			uxt: <Block as BlockT>::Extrinsic,
			len: u32,
		) -> pallet_transaction_payment_rpc_runtime_api::RuntimeDispatchInfo<Balance> {
			TransactionPayment::query_info(uxt, len)
		}
	}

	impl pallet_staking_rpc_runtime_api::StakingApi<Block, AccountId, Balance> for Runtime {
		fn staking_module_account_id() -> AccountId {
			Staking::account_id()
		}

		fn pool_account_id(id: u32) -> AccountId {
			Staking::pool_account_id(id)
		}

		fn pending_rewards(pool_id: u32, account_id: AccountId) -> Balance {
			Staking::pending_rewards(pool_id, account_id)
		}
	}
}
