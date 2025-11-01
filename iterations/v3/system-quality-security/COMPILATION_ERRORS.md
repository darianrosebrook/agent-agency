# system-quality-security - Compilation Errors

**Status**: 5 compilation errors blocking build

## Summary

This crate has 5 compilation errors that prevent it from building:
1. Missing `rand_distr::Laplace` import (2 errors)
2. Type annotation needed for `BoundKey` (2 errors)
3. Borrow checker issue with key rotation (1 error)

---

## Error 1: Missing `rand_distr::Laplace` Import (2 occurrences)

### Location
- `src/privacy_anonymization.rs:207`
- `src/privacy_anonymization.rs:471`

### Error Message
```
error[E0432]: unresolved import `rand_distr::Laplace`
   --> system-quality-security/src/privacy_anonymization.rs:207:26
    |
207 |         use rand_distr::{Laplace, Distribution};
    |                          ^^^^^^^ no `Laplace` in the root
```

### Context
The `Laplace` distribution is being used for differential privacy anonymization, but the type doesn't exist in `rand_distr` or isn't enabled as a feature.

### ✅ Solution Found in Existing Codebase
**Location**: `iterations/v3/system-federated-ml/src/differential_privacy.rs:95-116`

The `system-federated-ml` crate already implements Laplace noise generation manually using `Uniform` distribution and logarithmic transformation. The `rand_distr::Laplace` type doesn't exist in the crate.

### Fix Required
Replace `rand_distr::Laplace` usage with manual Laplace distribution implementation:

```rust
// Instead of:
use rand_distr::{Laplace, Distribution};
let noise_dist = Laplace::new(0.0, scale)?;

// Use manual implementation (from system-federated-ml):
use rand::Rng;
let mut rng = rand::thread_rng();
// Sample from Laplace distribution: (1/(2b)) * exp(-|x|/b)
// Can be generated as: sign * b * ln(U) where U ~ Uniform(0,1)
let u: f64 = rng.gen();
let sign = if rng.gen_bool(0.5) { 1.0 } else { -1.0 };
let noise = sign * scale * (1.0 - u).ln();
```

### Files to Check
- `src/privacy_anonymization.rs:207` - Replace Laplace import with manual implementation
- `src/privacy_anonymization.rs:471` - Remove unused Laplace import
- Reference: `iterations/v3/system-federated-ml/src/differential_privacy.rs:95-116` for complete implementation

---

## Error 2: Type Annotation Needed for `BoundKey` (2 occurrences)

### Location
- `src/data_encryption.rs:290`
- `src/data_encryption.rs:370`

### Error Message
```
error[E0282]: type annotations needed
   --> system-quality-security/src/data_encryption.rs:290:13
    |
290 |         let mut sealing_key = BoundKey::new(unbound_key, SimpleNonceSequence::new(nonce_bytes));
    |             ^^^^^^^^^^^^^^^
...
296 |         sealing_key.seal_in_place_append_tag(nonce, aad, &mut in_out)
    |         ----------- type must be known at this point
```

### Context
`BoundKey` is a generic type that requires explicit type parameters. The compiler cannot infer the concrete type from context.

### Fix Required
Add explicit type annotations using `SealingKey` for encryption and `OpeningKey` for decryption:

**For encryption (line 290):**
```rust
use ring::aead::{SealingKey, OpeningKey};
let mut sealing_key: BoundKey<SealingKey<AES_256_GCM>> = 
    BoundKey::new(unbound_key, SimpleNonceSequence::new(nonce_bytes));
```

**For decryption (line 370):**
```rust
let mut opening_key: BoundKey<OpeningKey<AES_256_GCM>> = 
    BoundKey::new(unbound_key, SimpleNonceSequence::new(nonce_array));
```

**Note**: The `ring` crate uses `SealingKey` for encryption operations and `OpeningKey` for decryption operations. Both are parameterized by the algorithm type (`AES_256_GCM`).

### Files to Check
- `src/data_encryption.rs:9` - Add `SealingKey, OpeningKey` to imports
- `src/data_encryption.rs:290` - Add type annotation for sealing key
- `src/data_encryption.rs:370` - Add type annotation for opening key

---

## Error 3: Cannot Move Out of Borrowed Value

### Location
- `src/data_encryption.rs:402`

### Error Message
```
error[E0505]: cannot move out of `manager` because it is borrowed
   --> system-quality-security/src/data_encryption.rs:402:14
    |
396 |         let mut manager = self.key_manager.write().await;
    |             ----------- binding `manager` declared here
397 |         
398 |         let old_key = manager.keys.get(&key_id)
    |                       ------- borrow of `manager` occurs here
...
402 |         drop(manager); // Release lock
    |              ^^^^^^^ move out of `manager` occurs here
403 |         let new_key_id = self.generate_key(old_key.algorithm, old_key.rotation_days).await?;
    |                                            ----------------- borrow later used here
```

### Context
The code attempts to:
1. Get a write lock on `manager`
2. Extract `old_key` (which borrows from `manager`)
3. Drop the lock
4. Use `old_key` after the lock is dropped

However, `old_key` is still borrowed from `manager`, so dropping `manager` would invalidate the borrow.

### ✅ Solution Pattern Found in Existing Codebase
**Location**: `iterations/v3/system-quality-security/src/keystore.rs:277-314`

The pattern of extracting values before dropping locks is used elsewhere in the codebase.

### Fix Required
Extract the needed values from `old_key` before dropping the lock:

```rust
let old_key = manager.keys.get(&key_id)
    .ok_or_else(|| EncryptionError::KeyNotFound { key_id })?;

// Extract owned values before dropping lock
let algorithm = old_key.algorithm;  // Copy type (Copy trait)
let rotation_days = old_key.rotation_days;  // Option<u32> (Copy if u32 is Copy)

drop(manager); // Release lock

let new_key_id = self.generate_key(algorithm, rotation_days).await?;
```

**Note**: Since `EncryptionAlgorithm` and `Option<u32>` are `Copy` types, they can be extracted without cloning. The borrow ends when values are copied.

### Files to Check
- `src/data_encryption.rs:395-410` - `rotate_key` method
- Reference: `iterations/v3/system-quality-security/src/keystore.rs:277-314` for similar pattern

---

## Recommended Fix Order

1. **Fix Error 3** (borrow checker) - Most straightforward, extract values before dropping lock
2. **Fix Error 1** (Laplace import) - Replace with manual implementation from `system-federated-ml`
3. **Fix Error 2** (type annotations) - Add `SealingKey`/`OpeningKey` type annotations for `BoundKey`

## ✅ Solutions Found in Existing Codebase

All three errors have solutions that can be derived from existing implementations:

1. **Laplace Distribution**: See `iterations/v3/system-federated-ml/src/differential_privacy.rs:95-116` for manual Laplace noise implementation
2. **BoundKey Type Annotations**: Ring crate requires `BoundKey<SealingKey<AES_256_GCM>>` and `BoundKey<OpeningKey<AES_256_GCM>>`
3. **Borrow Checker**: Extract `Copy` values before dropping lock (similar pattern in `src/keystore.rs:277-314`)

---

## Related Dependencies

**Current dependencies:**
- `rand_distr = "0.5"` - Laplace distribution not available (use manual implementation)
- `ring` - Requires `SealingKey` and `OpeningKey` imports for type annotations
- `rand` - Already available, used for manual Laplace implementation

**No dependency changes needed** - All fixes use existing dependencies.

