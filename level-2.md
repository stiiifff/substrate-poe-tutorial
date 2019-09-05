# Substrate Proof of Existence Module

This is a simple Substrate runtime module to store online distributed [proof of existence](https://www.proofofexistence.com/) for any file.

# Tutorial - Level 2

In our previous iterations, we gradually built a simple proof-of-existence mechanism that allows to store, query & erase digital proofs on-chain. This is fine for a feature exposed in a closed consortium Blockchain where we assume participants are known and not ill-intentioned: indeed, a limitation of the current implementation is that it does not implement any anti-spam feature. An ill-intentioned participant could mount an attack on the chain by sending a lot of transactions aimed at registering a very high number of (actual or bogus) digests, thereby causing the nodes to consume valuable computing resources (and potentially prevent other more valuable chain features to be served to other users in the ecosystem).

To prevent these situations, user-accessible features exposed via on-chain logic (smart contracts, or Substrate’s runtime modules) typically integrate some economic disincentives (negative incentives) to prevent abuses. This is exactly what we’re going to do at this stage.
We will add a simple fee mechanism that will ask users who desire to store a proof on-chain to deposit a certain fee that will remain locked for as long as the proof is being recorded.
To do so, we will leverage generic traits that are part of Substrate: the [Currency](https://crates.parity.io/srml_support/traits/trait.Currency.html) and [ReservableCurrency](https://crates.parity.io/srml_support/traits/trait.ReservableCurrency.html) support traits.

Additionally, to better reflect the intent of our runtime module, we will adapt the **terminology** that we use in function names: we will now talk about **creating a claim** and **revoking a claim** on a particular proof instead of **storing** / **erasing** (which did sound a bit too database-CRUDish).

First, we'll starting by referencing the traits we need in our module, and declaring a **Currency type** in our module's configuration trait. We also take the opportunity to declare a **BalanceOf** alias type that will later make our code more readable:
```rust
use support::traits::{Currency, ReservableCurrency};

// Shorthand type for Balance type from Currency trait
type BalanceOf<T> = <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance;

pub trait Trait: timestamp::Trait {
  type Currency: ReservableCurrency<Self::AccountId>;

  // ...
}
```

Next, to align with aforementionned terminology of claiming and revoking a proof, we rename the module's events:
```rust
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
```

We can then revisit the logic of our **store_proof** and **erase_proof** functions, that we rename to **create_claim** and **revoke_claim**. We also define a **fixed fee** as a constant that will act as (very basic) [economic disincentive](https://en.wikipedia.org/wiki/Disincentive) to mitigate abuse of our proof-of-existence feature: this amount will be **reserved in the sender's account balance** when a claim is created, and **released when the claim is later revoked**:
```rust
// ...
use system::ensure_signed;

const POE_FEE: u64 = 1000;

// ...

decl_module! {
  pub struct Module<T: Trait> for enum Call where origin: T::Origin {
    // ...

    // This function can be called by the external world as an extrinsics call.
    // The origin parameter is of type `AccountId`.
    // The function performs a few verifications, then stores the proof and emits an event.
    fn create_claim(origin, digest: Vec<u8>) -> Result {
      // Verify that the incoming transaction is signed
      let sender = ensure_signed(origin)?;

      // Validate digest does not exceed a maximum size
      ensure!(digest.len() <= DIGEST_MAXSIZE, ERR_DIGEST_TOO_LONG);

      // Verify that the specified proof has not been claimed yet
      ensure!(!Proofs::<T>::exists(&digest), "This proof has already been claimed");
      // Get current time for current block using the base timestamp module
      let time = timestamp::Module::<T>::now();

      // Reserve the fee in the sender's account balance
      T::Currency::reserve(&sender, BalanceOf::<T>::from(POE_FEE))?;

      // Store the proof and the sender of the transaction, plus block time
      Proofs::<T>::insert(&digest, (sender.clone(), time.clone()));

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

      // Verify that the specified proof has been claimed before
      ensure!(Proofs::<T>::exists(&digest), "This proof has not been claimed yet");

      // Get owner associated with the proof
      let (owner, _time) = Self::proofs(&digest);

      // Verify that sender of the current tx is the proof owner
      ensure!(sender == owner, "You must own this claim to revoke it");

      // Erase proof from storage
      Proofs::<T>::remove(&digest);

      // Release previously reserved fee from owner's account balance
      T::Currency::unreserve(&sender, BalanceOf::<T>::from(POE_FEE));

      // Issue an event to notify that the claim was effectively revoked
      Self::deposit_event(RawEvent::ClaimRevoked(sender, digest));

      Ok(())
    }
  }
}
```

Last step, we need update our test code to account for the additional traits implemented, and to additional checks to verify that a claim holder has her account balance impacted by the fee mechanism !
```rust
// ADD THIS
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

// ...

impl Trait for Test {
  type Event = ();
  // ADD THIS
  type Currency = balances::Module<Test>;
}
// ADD THIS
type Balances = balances::Module<Test>;

// MODIFY THE FUNCTION USED TO DEFINE A MOCK STORAGE WITH 2 ACCOUNTS AND BALANCES
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

// AND PRETTY MUCH REVAMP THE TESTS :)
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
```

Once you've got all that in place, it's time to run the tests again and verify everything is running smoothly !

# The end

You now know how to build a **custom Substrate runtime module**, by declaring **storage items**, **events**, defining **dispatchable functions**, and writing **tests** for these. You also know how to leverage existing features from built-in runtime modules like the **timestamp** module or generic traits like the **Currency** trait.

Last but not least, you know how to compile your **custom Substrate node** that includes your custom module in the **runtime**, and run it locally.

Congratulations ! You are now ready to advance your Substrate development skills to further levels ! Go on exploring the [Substrate Developer Hub](https://substrate.dev) and continue expanding your knowledge !
