# Substrate Proof of Existence Module

This is a simple Substrate runtime module to store online distributed [proof of existence](https://www.proofofexistence.com/) for any file.

# Tutorial - Level 1

In its first iteration ([level-0](level-0.md)), we kept the PoE runtime module as simple as possible on purpose.
We will now revisit one of its shortcomings, namely that the proofs are only associated with their owner, there is not timing information. This implies that when querying for a proof, we know if it exists or not, and which account created it, but we don’t know when. We could issue additional commands to find out in which block the transaction that led to the creation / erasure was mined, but this is not very convenient from a usability perspective.

What we need is to record the time at which a proof was created along with its owner.
To do so, we will learn how to leverage existing features from standard [built-in Substrate runtime modules](https://substrate.dev/docs/en/runtime/substrate-runtime-module-library#srml-modules). In this particular case, we will use the [timestamp module](https://substrate.dev/rustdocs/v1.0/srml_timestamp/index.html).

To access the block time in our module's logic, we must **derive our module's configuration trait from the timestamp trait**:
```rust
//REPLATE THIS: pub trait Trait: system::Trait
pub trait Trait: timestamp::Trait {
  // ...
}
```
Previously, our module configuration trait was deriving from the [system module](https://substrate.dev/rustdocs/v1.0/srml_system/index.html) trait which is the base for all runtime modules.

Next, we need to revisit the **Proofs map** used to store proofs, to use a tuple of AccountId plus block time as value, instead of just the AccountId:
```rust
decl_storage! {
  trait Store for Module<T: Trait> as PoeStorage {

    //STORE TUPLE OF (AccountID, Moment) AS VALUE IN MAP
    Proofs get(proofs): map Vec<u8> => (T::AccountId, T::Moment);
  }
}
```
From the timestamp module, we have access to the [Moment type](https://substrate.dev/rustdocs/v1.0/srml_timestamp/trait.Trait.html#associatedtype.Moment) that is used to express a timestamp.

Additionnally, we update the **ProofStored** event definition to add the timestamp:
```rust
decl_event!(
  pub enum Event<T> where
    AccountId = <T as system::Trait>::AccountId,
    //DON't FORGET THIS
    Moment = <T as timestamp::Trait>::Moment
   {
    //ADD Moment timestamp
    ProofStored(AccountId, Moment, Vec<u8>),

    // ...
  }
);
```

We can then modify our **store_proof** and **erase_proof** functions to account for block time:
```rust
fn store_proof(origin, digest: Vec<u8>) -> Result {

  // Verify that the incoming transaction is signed
  let sender = ensure_signed(origin)?;

  // Validate digest does not exceed a maximum size
  ensure!(digest.len() <= DIGEST_MAXSIZE, ERR_DIGEST_TOO_LONG);

  // Verify that the specified proof has not been stored yet
  ensure!(!Proofs::<T>::exists(&digest), "This proof has already been stored");

  // ADD THIS
  // Get current time for current block using the base timestamp module
  let time = timestamp::Module::<T>::now();

  // Store the proof and the sender of the transaction, plus block time
  Proofs::<T>::insert(&digest, (sender.clone(), time.clone())); // <- ADD time.clone()

  // Issue an event to notify that the proof was successfully stored
  Self::deposit_event(RawEvent::ProofStored(sender, time, digest)); // <- ADD time

  Ok(())
}

fn erase_proof(origin, digest: Vec<u8>) -> Result {

  // Verify that the incoming transaction is signed
  let sender = ensure_signed(origin)?;

  // Validate digest does not exceed a maximum size
  ensure!(digest.len() <= DIGEST_MAXSIZE, ERR_DIGEST_TOO_LONG);

  // Verify that the specified proof has been stored before
  ensure!(Proofs::<T>::exists(&digest), "This proof has not been stored yet");

  // DESTRUCTURED ASSIGNMENT TO (owner, _time)
  // Get owner associated with the proof
  let (owner, _time) = Self::proofs(&digest);

  // Verify that sender of the current tx is the proof owner
  ensure!(sender == owner, "You must own this proof to erase it");

  // Erase proof from storage
  Proofs::<T>::remove(&digest);

  // Issue an event to notify that the proof was effectively erased
  Self::deposit_event(RawEvent::ProofErased(sender, digest));

  Ok(())
}
```

Finally, we also must update our test code to add mocks for the timestamp module's trait:
```rust
impl system::Trait for Test {
  // ...
}

//ADD THE FOLLOWING
parameter_types! {
  pub const MinimumPeriod: u64 = 5;
}
impl timestamp::Trait for Test {
  type Moment = u64;
  type OnTimestampSet = ();
  type MinimumPeriod = MinimumPeriod;
}
//--

impl Trait for Test {
  type Event = ();
}
type POEModule = Module<Test>;
```
The rest of our test code remains unchanged.

With all that in place, we can now run the test, all they should still pass !
___

In this step of the tutorial, we have learned how to leverage functionalities from existing Substrate runtime modules by deriving our module configuration trait. In the next and final step of the tutorial, we will see how to integrate even more with existing modules. Let's advance to level-2 !
