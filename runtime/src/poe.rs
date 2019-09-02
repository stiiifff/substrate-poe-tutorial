/// A runtime module for a simple Proof-of-existence mechanism.

use support::{decl_module, decl_storage, decl_event, ensure, StorageMap, dispatch::Result};
use rstd::vec::Vec;
use system::ensure_signed;

/// The module's configuration trait.
pub trait Trait: system::Trait {
    /// The overarching event type.
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

// This module's storage items.
decl_storage! {
	trait Store for Module<T: Trait> as PoeStorage {
        // Define a 'Proofs' storage space for a map with
        // the proof digest as the key, and associated AccountId as value.
        // The 'get(proofs)' is the default getter.
		Proofs get(proofs): map Vec<u8> => T::AccountId;
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
		fn store_proof(origin, digest: Vec<u8>) -> Result {
            // Verify that the incoming transaction is signed
            let sender = ensure_signed(origin)?;

            // Verify that the specified proof has not been stored yet
            ensure!(!<Proofs<T>>::exists(&digest), "This proof has already been stored");

            // Store the proof and the sender of the transaction
            <Proofs<T>>::insert(&digest, sender.clone());

            // Issue an event to notify that the proof was successfully stored
            Self::deposit_event(RawEvent::ProofStored(sender, digest));

            Ok(())
        }

        // This function's structure is similar to the store_proof function.
        // The function performs a few verifications, then erase an existing proof from storage,
        // and finally emits an event.
		fn erase_proof(origin, digest: Vec<u8>) -> Result {
            // Verify that the incoming transaction is signed
            let sender = ensure_signed(origin)?;
            
            // Verify that the specified proof has been stored before
            ensure!(<Proofs<T>>::exists(&digest), "This proof has not been stored yet");

            // Get owner associated with the proof
            let owner = Self::proofs(&digest);

            // Verify that sender of the current tx is the proof owner
            ensure!(sender == owner, "You must own this proof to erase it");

            // Erase proof from storage
            <Proofs<T>>::remove(&digest);

            // Issue an event to notify that the proof was effectively erased
            Self::deposit_event(RawEvent::ProofErased(sender, digest));

            Ok(())
        }
	}
}

// This module's events.
decl_event!(
	pub enum Event<T> where AccountId = <T as system::Trait>::AccountId {
        // Event emitted when a proof has been stored into chain storage
		ProofStored(AccountId, Vec<u8>),
        // Event emitted when a proof has been erased from chain storage
		ProofErased(AccountId, Vec<u8>),
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
	impl Trait for Test {
		type Event = ();
	}
	type POEModule = Module<Test>;

	// This function basically just builds a genesis storage key/value store according to
	// our desired mockup.
	fn new_test_ext() -> runtime_io::TestExternalities<Blake2Hasher> {
		system::GenesisConfig::default().build_storage::<Test>().unwrap().into()
	}

	#[test]
	fn it_works_for_default_value() {
		with_externalities(&mut new_test_ext(), || {

            // Have account 1 stores a proof
			assert_ok!(POEModule::store_proof(Origin::signed(1), vec![0]));
            // Check that account 2 cannot create the same proof
            assert_noop!(POEModule::store_proof(Origin::signed(2), vec![0]), "This proof has already been stored");
            // Check that account 2 cannot erase a proof they do not own
            assert_noop!(POEModule::erase_proof(Origin::signed(2), vec![0]), "You must own this proof to erase it");
            // Check that account 2 cannot revoke some non-existent proof
            assert_noop!(POEModule::erase_proof(Origin::signed(2), vec![1]), "This proof has not been stored yet");
            // Check that account 1 can erase their proof
            assert_ok!(POEModule::erase_proof(Origin::signed(1), vec![0]));
            // Check that account 2 can now store this proof
            assert_ok!(POEModule::store_proof(Origin::signed(2), vec![0]));
		});
	}
}
