/// A runtime module for a simple Proof-of-existence mechanism.

use support::{decl_module, decl_storage, decl_event, ensure, StorageMap, dispatch::Result};
use support::traits::{Currency, ReservableCurrency};
use rstd::vec::Vec;
use system::ensure_signed;

pub const ERR_DIGEST_TOO_LONG: &str = "Digest too long (max 100 bytes)";
pub const DIGEST_MAXSIZE: usize = 100;

// Fee that users are supposed to deposit to
// hold a claim on a specific proof digest
const POE_FEE: u32 = 1000;

// Shorthand type for Balance type from Currency trait
type BalanceOf<T> = <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance;

/// The module's configuration trait.
pub trait Trait: timestamp::Trait {
	type Currency: ReservableCurrency<Self::AccountId>;
    /// The overarching event type.
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

// This module's storage items.
decl_storage! {
	trait Store for Module<T: Trait> as PoeStorage {
        // Define a 'Proofs' storage space for a map with
        // the proof digest as the key, and associated AccountId as value.
        // The 'get(proofs)' is the default getter.
		Proofs get(proofs): map Vec<u8> => (T::AccountId, T::Moment);
	}
}

// The module's dispatchable functions.
decl_module! {
    /// The module declaration.
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        // Initializing events
		// this is needed only if you are using events in your module
		fn deposit_event() = default;

		// This function can be called by the external world as an extrinsics call.
		// The origin parameter is of type `AccountId`.
        // The function performs a few verifications, then stores the proof and emits an event.
		fn create_claim(origin, digest: Vec<u8>) -> Result {
            // Verify that the incoming transaction is signed
            let sender = ensure_signed(origin)?;

			// Validate digest does not exceed a maximum size
			ensure!(digest.len() <= DIGEST_MAXSIZE, ERR_DIGEST_TOO_LONG);

            // Verify that the specified proof has not been claimed yet
            ensure!(!<Proofs<T>>::exists(&digest), "This proof has already been claimed");
			// Get current time for current block using the base timestamp module
			let time = <timestamp::Module<T>>::now();

			// Reserve the fee in the sender's account balance
			T::Currency::reserve(&sender, BalanceOf::<T>::from(POE_FEE))?;

            // Store the proof and the sender of the transaction, plus block time
            <Proofs<T>>::insert(&digest, (sender.clone(), time.clone()));

            // Issue an event to notify that the proof was successfully claimed
            Self::deposit_event(RawEvent::ClaimCreated(sender, time, digest));

            Ok(())
        }

        // This function's structure is similar to the store_proof function.
        // The function performs a few verifications, then revoke an existing proof from storage,
        // and finally emits an event.
		fn revoke_claim(origin, digest: Vec<u8>) -> Result {
            // Verify that the incoming transaction is signed
            let sender = ensure_signed(origin)?;

			// Validate digest does not exceed a maximum size
			ensure!(digest.len() <= DIGEST_MAXSIZE, ERR_DIGEST_TOO_LONG);

            // Verify that the specified proof has been claimed before
            ensure!(<Proofs<T>>::exists(&digest), "This proof has not been claimed yet");

            // Get owner associated with the proof
            let (owner, _time) = Self::proofs(&digest);

            // Verify that sender of the current tx is the proof owner
            ensure!(sender == owner, "You must own this claim to revoke it");

            // Erase proof from storage
            <Proofs<T>>::remove(&digest);

			// Release previously reserved fee from owner's account balance
			T::Currency::unreserve(&sender, BalanceOf::<T>::from(POE_FEE));

            // Issue an event to notify that the claim was effectively revoked
            Self::deposit_event(RawEvent::ClaimRevoked(sender, digest));

            Ok(())
        }
	}
}

// This module's events.
decl_event!(
	pub enum Event<T> where
		AccountId = <T as system::Trait>::AccountId,
		Moment = <T as timestamp::Trait>::Moment
	 {
        // Event emitted when a proof has been successfully claimed
		ClaimCreated(AccountId, Moment, Vec<u8>),
        // Event emitted when a proof claim has been revoked
		ClaimRevoked(AccountId, Vec<u8>),
	}
);

// tests for this module
#[cfg(test)]
mod tests {
	use super::*;

	use runtime_io::with_externalities;
	use primitives::{H256, Blake2Hasher};
	use support::{impl_outer_origin, assert_ok, assert_noop, parameter_types};
	use sr_primitives::{traits::{BlakeTwo256, IdentityLookup}, testing::Header};
	use sr_primitives::weights::Weight;
	use sr_primitives::Perbill;

	impl_outer_origin! {
		pub enum Origin for Test {}
	}

	// For testing the module, we construct most of a mock runtime. This means
	// first constructing a configuration type (`Test`) which `impl`s each of the
	// configuration traits of modules we want to use.
	#[derive(Clone, Eq, PartialEq)]
	pub struct Test;
	parameter_types! {
		pub const BlockHashCount: u64 = 250;
		pub const MaximumBlockWeight: Weight = 1024;
		pub const MaximumBlockLength: u32 = 2 * 1024;
		pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
	}
	impl system::Trait for Test {
		type Origin = Origin;
		type Call = ();
		type Index = u64;
		type BlockNumber = u64;
		type Hash = H256;
		type Hashing = BlakeTwo256;
		type AccountId = u64;
		type Lookup = IdentityLookup<Self::AccountId>;
		type Header = Header;
		type WeightMultiplierUpdate = ();
		type Event = ();
		type BlockHashCount = BlockHashCount;
		type MaximumBlockWeight = MaximumBlockWeight;
		type MaximumBlockLength = MaximumBlockLength;
		type AvailableBlockRatio = AvailableBlockRatio;
		type Version = ();
	}
	impl balances::Trait for Test {
		type Balance = u64;
		type OnNewAccount = ();
		type OnFreeBalanceZero = ();
		type Event = ();
		type TransactionPayment = ();
		type TransferPayment = ();
		type DustRemoval = ();
		type ExistentialDeposit = ();
		type TransferFee = ();
		type CreationFee = ();
		type TransactionBaseFee = ();
		type TransactionByteFee = ();
		type WeightToFee = ();
    }
	parameter_types! {
		pub const MinimumPeriod: u64 = 5;
	}
	impl timestamp::Trait for Test {
        type Moment = u64;
        type OnTimestampSet = ();
		type MinimumPeriod = MinimumPeriod;
    }
	impl Trait for Test {
		type Event = ();
		type Currency = balances::Module<Test>;
	}
	type Balances = balances::Module<Test>;
	type POEModule = Module<Test>;

	// This function basically just builds a genesis storage key/value store according to
	// our desired mockup.
	fn new_test_ext() -> runtime_io::TestExternalities<Blake2Hasher> {
		let mut t = system::GenesisConfig::default().build_storage::<Test>().unwrap();
		balances::GenesisConfig::<Test> {
			balances: vec![(1, 10000), (2, 10000)],
			vesting: vec![],
		}.assimilate_storage(&mut t).unwrap();
        t.into()
	}

	#[test]
	fn it_works() {
		with_externalities(&mut new_test_ext(), || {

			// Verify it's not possible to store exceedingly big digests (prevent DOS attack and/or chain storage bloat)
			assert_noop!(POEModule::create_claim(Origin::signed(1), vec![0; 101]), "Digest too long (max 100 bytes)");

            // Have account 1 create a claim
			assert_ok!(POEModule::create_claim(Origin::signed(1), vec![0]));

			// Check that account 1 reserved their deposit for creating a claim
            assert_eq!(Balances::free_balance(&1), 9000);
            assert_eq!(Balances::reserved_balance(&1), 1000);

            // Check that account 2 cannot create the same claim
            assert_noop!(POEModule::create_claim(Origin::signed(2), vec![0]), "This proof has already been claimed");
            // Check that account 2 cannot revoke a claim they do not own
            assert_noop!(POEModule::revoke_claim(Origin::signed(2), vec![0]), "You must own this claim to revoke it");
            // Check that account 2 cannot revoke some non-existent claim
            assert_noop!(POEModule::revoke_claim(Origin::signed(2), vec![1]), "This proof has not been claimed yet");

            // Check that account 1 can revoke their claim
            assert_ok!(POEModule::revoke_claim(Origin::signed(1), vec![0]));

			// Check that account 1 got back their deposit
            assert_eq!(Balances::free_balance(&1), 10000);
            assert_eq!(Balances::reserved_balance(&1), 0);

            // Check that account 2 can now claim this digest
            assert_ok!(POEModule::create_claim(Origin::signed(2), vec![0]));
		});
	}
}
