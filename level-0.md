# Substrate Proof of Existence Module

This is a simple Substrate runtime module to store online distributed [proof of existence](https://www.proofofexistence.com/) for any file.

# Tutorial - Level 0

Our first **Substrate runtime module** will allow users & applications to store & retrieve digital fingerprint of files. This is a process known as Anchoring or "Proof-of-existence", as in, in the case of a dispute in a contractual relation or regarding the ownership of an asset, a person is able to prove that a certain document with a specific content existed at a certain point in time therefore "proving" the author's or owner's point.

To implement an anchoring mechanism, our module must be able to:
- store the hash (or digest) of a file's content, submitted as part of a transaction.
- allow to retrieve a hash, to verify its existence on chain.

To keep things simple, no fees are involved at this stage (this will be for the level-2 step of the tutorial).

___

## 1. Setting up the project

In this first step, there are 2 ways to do it:

- **Quick way**: clone this repository's **level-0** branch to get started directly. You can then skip ahead to [point 2](#2-building-and-renaming-the-template-module).
- **Recommended way**: Start fresh from the [substrate-node-template](https://github.com/substrate-developer-hub/substrate-node-template) repository. Follow the instructions below.


Clone the [substrate-node-template](https://github.com/substrate-developer-hub/substrate-node-template) repo into a local folder:
```bash
git clone --depth=1 https://github.com/substrate-developer-hub/substrate-node-template.git my_project
cd my_project
```

Use the bash script to rename the template to your project name:
```bash
chmod u+x ./substrate-node-rename.sh
./substrate-node-rename.sh "my_project" "author"
````

This should give you the following output:
```bash
Moving project folder...
Customizing project...
Rename Complete
```
The template has now been renamed to your very own project name, and we can get to work !

Optionally, but recommended:
Move things around a bit & create a fresh git repo:
```bash
cd my_project
rm LICENSE README.md substrate-node-rename.sh substrate-node-template.tar.gz
mv my_project/* ./ && rm -r my_project/ && rm -rf .git/
git init . && git add . && git commit -a -m "Initial commit"
```

___

## 2. Building and renaming the template module

Now that you've got a clean runtime module project, let's first make sure you can build it:

Ensure your have the required dependencies installed on your machine (Rust, etc.):
```basj
curl https://sh.rustup.rs -sSf | sh
````

Install required tools
```bash
./scripts/init.sh
````

Build the project (Was and native code)
```bash
cargo build
````

To start a single-node development chain:
```bash
cargo run — —dev
```

As part of the template, we have a generic module located at **runtime/src/template.rs**

Let's modify it to build our own proof-of-existence runtime module. First, we'll rename the module:
1. Rename **runtime/src/template.rs** to **runtime/src/poe.rs**:
```bash
mv runtime/src/template.src runtime/src/poe.rs
```

2. In **runtime/src/lib.rs**, perform the following replacements:

```rust
//REMOVE THIS: mod template;
mod poe;

//...

//REMOVE THIS: impl template::Trait for Runtime {
impl poe::Trait for Runtime {
	//...
}

construct_runtime!(
	//...

	//REMOVE THIS: TemplateModule: template::{Module, Call, Storage, Event<T>},
	Poe: poe::{Module, Call, Storage, Event<T>},

	//...
);
```

If you got it right, the project should still build correctly !

___

## 3. Implementing the runtime module's logic

Head over to the **runtime/src/poe.rs** file that contains our runtime module's code.

Using the [decl_storage! macro](https://substrate.dev/docs/en/runtime/types/module-struct#runtime-storage), we define storage items, that is, data structures that will be stored on-chain. Let's add a **map** (Substrate's hashtable) to store file fingerprints and their associated owner. For this, we need a **StorageMap** so don't forget to add it to the **use::support instruction** above.
```rust
use support::{decl_storage, StorageMap};
use rstd::vec::Vec;

pub const ERR_DIGEST_TOO_LONG: &str = "Digest too long (max 100 bytes)";
pub const DIGEST_MAXSIZE: usize = 100;

decl_storage! {
    trait Store for Module<T: Trait> as PoeStorage {

		// Define a 'Proofs' storage space for a map with
		// the proof digest as the key, and associated AccountId as value.
		// The 'get(proofs)' is the default getter.
        Proofs get(proofs): map Vec<u8> => T::AccountId;
    }
}
```

Before proceeding to defining the functions that will implement the logic of our proof-of-existence system, we need to declare the events that will be raised whenever a proof was stored or erased. We add them using the [decl_event! macro](https://substrate.dev/docs/en/runtime/types/event-enum):
```rust
use support::{decl_event};

decl_event!(
    pub enum Event<T> where AccountId = <T as system::Trait>::AccountId {

		// Event emitted when a proof has been stored into chain storage
        ProofStored(AccountId, Vec<u8>),
		// Event emitted when a proof has been erased from chain storage
        ProofErased(AccountId, Vec<u8>),
    }
);
```

Finally we can implement the module's logic, exposed to external apps / systems as dispatchable functions, using the [decl_module! macro](https://substrate.dev/docs/en/runtime/macros/decl_module):
- The **store_proof** function will record a proof digest on-chain along with the Account that initiated the transaction for the call (after ensuring the call is valid, and the proof is not stored yet).
- The **erase_proof** function will suppress a pproof digest from on-chain storage, after having verified that the proof is known and the sending Account is effectively the owner of that proof.

```rust
use support::{decl_module, ensure, dispatch::Result};
use system::ensure_signed;

// The module's dispatchable functions.
decl_module! {
	/// The module declaration.
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		// Initializing events
		// this is needed only if you are using events in your module
		fn deposit_event() = default;

		fn store_proof(origin, digest: Vec<u8>) -> Result {

			// Verify that the incoming transaction is signed
			let sender = ensure_signed(origin)?;

			// Validate digest does not exceed a maximum size
			ensure!(digest.len() <= DIGEST_MAXSIZE, ERR_DIGEST_TOO_LONG);

			// Verify that the specified proof has not been stored yet
			ensure!(!Proofs::<T>::exists(&digest), "This proof has already been stored");

			// Store the proof and the sender of the transaction
			Proofs::<T>::insert(&digest, sender.clone());

			// Issue an event to notify that the proof was successfully stored
			Self::deposit_event(RawEvent::ProofStored(sender, digest));

			Ok(())
		}

		fn erase_proof(origin, digest: Vec<u8>) -> Result {

			// Verify that the incoming transaction is signed
			let sender = ensure_signed(origin)?;

			// Validate digest does not exceed a maximum size
			ensure!(digest.len() <= DIGEST_MAXSIZE, ERR_DIGEST_TOO_LONG);

			// Verify that the specified proof has been stored before
			ensure!(Proofs::<T>::exists(&digest), "This proof has not been stored yet");

			// Get owner associated with the proof
			let owner = Self::proofs(&digest);

			// Verify that sender of the current tx is the proof owner
			ensure!(sender == owner, "You must own this proof to erase it");

			// Erase proof from storage
			Proofs::<T>::remove(&digest);

			// Issue an event to notify that the proof was effectively erased
			Self::deposit_event(RawEvent::ProofErased(sender, digest));

			Ok(())
		}
	}
}
```

To verify that the module is building correctly, we first comment out everything enclosed in the
**mod tests** at the bottom of the file,
```rust

/* COMMENT EVERYTHING BELOW FOR NOW */

/// tests for this module
//#[cfg(test)]
//mod tests {
//    ...
//}
```

and then run:
```bash
cargo build
```

___

### 4. Writing tests to verify the module's logic

Now that we have implemented our basic proof-of-existence runtime module, we will write unit tests to verify the logic written in the module's dispatchable functions behave as expected.
(First uncomment the lines we previously commented out ...)

```
#[cfg(test)]
mod tests {
	use support::{impl_outer_origin, assert_ok, assert_noop, parameter_types};

	// ...

	//REMOVE THIS: type TemplateModule = Module<Test>;
	type POEModule = Module<Test>;

	// ...

	#[test]
	fn it_works() {
		with_externalities(&mut new_test_ext(), || {

			// Verify it's not possible to store exceedingly big digests (prevent DOS attack and/or chain storage bloat)
			assert_noop!(POEModule::store_proof(Origin::signed(1), vec![0; 101]), "Digest too long (max 100 bytes)");

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

```

To execute the tests, either run:
```bash
cargo test -p my-project-runtime template
```
or
```bash
cd runtime
cargo test
```

If all goes well, you should get the following output:
```bash
running 1 test
test poe::tests::it_works ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

If you successfully reached this point, congratulations, you just wrote your first fully-functional Substrate runtime module !!!

You can now proceed to [level-1](level-1.md) of the tutorial.
