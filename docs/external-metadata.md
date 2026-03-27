# External Metadata Decentralized Storage Link

This feature allows the SoroMint token to store a reference to external rich metadata (such as logos, descriptions, or legal documents) stored on decentralized storage platforms like IPFS or Arweave.

## Overview

Tokens often need more metadata than can be efficiently stored directly on the blockchain. By storing a content hash (CID for IPFS, or Arweave transaction ID), the token remains lightweight while still providing a verifiable link to external resources.

## Implementation Details

### Functions

#### `set_metadata_hash(e: Env, hash: String)`
Sets the metadata hash for the token.
- **Authorization**: Only the contract administrator (admin) can call this function.
- **Args**: `hash` is a `String` representing the IPFS/Arweave hash.
- **Events**: Emits a `metadata_updated` event.

#### `metadata_hash(e: Env) -> Option<String>`
Retrieves the current metadata hash.
- **Returns**: `Some(hash)` if a hash has been set, otherwise `None`.

### Security Considerations

- **Admin Only**: The update function is protected by `admin.require_auth()`, ensuring that only the designated owner can change the metadata reference.
- **Storage Efficiency**: The hash is stored in the contract's instance storage (`DataKey::MetadataHash`), which is efficient for Soroban's storage model.
- **Immutability of Reference**: While the reference can be updated by the admin, the content pointed to by a hash (especially on IPFS) is content-addressed and thus immutable for that specific hash.

## Example Usage

### Setting a Metadata Hash (IPFS)
To link to a metadata file on IPFS with CID `QmXoypizjW3WknFiJnKLwHCnL72vedxjQkDDP1mXWo6uco`:
```rust
token.set_metadata_hash("QmXoypizjW3WknFiJnKLwHCnL72vedxjQkDDP1mXWo6uco");
```

### Retrieving the Hash
```rust
let hash = token.metadata_hash();
// Returns Some("QmXoypizjW3WknFiJnKLwHCnL72vedxjQkDDP1mXWo6uco")
```
