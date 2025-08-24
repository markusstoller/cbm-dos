# cbm-dos

A small Rust library that implements Commodore-style GCR (Group Code Recording) 4-to-5 encoding and decoding.

It converts 8-bit bytes into a stream of 5-bit codes ("quintuples") and back, using the classic 4-bit-to-5-bit mapping. The crate exposes a minimal API centered around the `GCR` type with `encode` and `decode` operations.

- 4-byte input encodes to 5 bytes of GCR output.
- 5-byte GCR input decodes to 4 bytes of original output.
- Invalid quintuples during decoding cause `decode` to return `None`.

## Status
- Version: 0.1.3
- Rust edition: 2024

## Why GCR (4-to-5)?
Group Code Recording was used on Commodore disk formats, mapping each 4-bit nibble to a 5-bit code that satisfies constraints for magnetic media. This library focuses on that mapping only (bit packing/unpacking and lookup), not on flux-level or disk image handling.

## Features
- Simple, allocation-friendly encoding/decoding routines
- O(1) lookups via precomputed tables
- Deterministic packing to/from 40-bit (5-byte) blocks
- Fully tested round-trip for a known vector

## Installation
Add the dependency to your `Cargo.toml`:

```toml
[dependencies]
cbm-dos = "0.1.3"
```

Then import in your code:

```rust
use cbm_dos::GCR;
```

## Quick start

```rust
use cbm_dos::GCR;

fn main() {
    // Create a GCR encoder/decoder
    let gcr = GCR::new();

    // Example: encode 8 bytes (must be a multiple of 4)
    let data: Vec<u8> = vec![0x08, 0x01, 0x00, 0x01, 0x30, 0x30, 0x00, 0x00];
    let encoded = gcr.encode(&data);
    assert_eq!(encoded, vec![0x52, 0x54, 0xB5, 0x29, 0x4B, 0x9A, 0xA6, 0xA5, 0x29, 0x4A]);

    // Decode back (input length must be a multiple of 5)
    let decoder = GCR::new();
    let decoded = decoder.decode(&encoded).expect("valid GCR");
    assert_eq!(decoded, data);
}
```

## API overview

- `GCR::new() -> GCR`
  - Constructs a new instance with precomputed encode/decode lookup tables.

- `GCR::encode(&self, input: &[u8]) -> Vec<u8>`
  - Encodes the input in chunks of 4 bytes at a time.
  - For each 4-byte chunk (8 nibbles), each nibble is mapped to a 5-bit code and packed into a 40-bit big-endian value, emitted as 5 bytes.
  - If `input.len()` is not a multiple of 4, extra bytes at the end are ignored. You should pad your input if you need exact coverage.

- `GCR::decode(&self, input: &[u8]) -> Option<Vec<u8>>`
  - Decodes the input in chunks of 5 bytes at a time.
  - Each 5-byte chunk is interpreted as a 40-bit big-endian value composed of 8 quintuples; each quintuple maps back to a 4-bit nibble.
  - Returns `None` if any quintuple in any chunk is invalid.

## The mapping

This library uses the canonical 16-entry mapping from 4-bit nibbles to 5-bit GCR codes (shown here as binary):

```
(Encoded -> Decoded nibble)
01010 -> 0x0
01011 -> 0x1
10010 -> 0x2
10011 -> 0x3
01110 -> 0x4
01111 -> 0x5
10110 -> 0x6
10111 -> 0x7
01001 -> 0x8
11001 -> 0x9
11010 -> 0xA
11011 -> 0xB
01101 -> 0xC
11101 -> 0xD
11110 -> 0xE
10101 -> 0xF
```

Internally the crate precomputes two lookup tables for O(1) translation:
- `decode_mappings[32]` indexed by the 5-bit code to obtain the 4-bit nibble (invalid entries are 0xFF)
- `encode_mappings[16]` indexed by the nibble to obtain its 5-bit code

## Input size rules and padding
- Encoding operates on exact 4-byte blocks. If the input length is not a multiple of 4, the trailing bytes are ignored. If you need to process all data, pad to a multiple of 4 and carry the padding information separately.
- Decoding operates on exact 5-byte blocks. If the input length is not a multiple of 5, the trailing bytes are ignored by the chunking iterator and will not be decoded. Provide complete 5-byte blocks.

## Error handling
- `decode` returns `None` if it encounters any 5-bit value that is not a valid GCR code (i.e., it maps to 0xFF in the internal table). This typically means the input stream is corrupted or misaligned.

## Example vectors
The tests in this crate include a round-trip sanity check:

```rust
use cbm_dos::GCR;

fn main() {
    let gcr = GCR::new();
    let encoded: Vec<u8> = vec![0x52, 0x54, 0xB5, 0x29, 0x4B, 0x9A, 0xA6, 0xA5, 0x29, 0x4A];
    let decoded = gcr.decode(&encoded).unwrap();
    assert_eq!(decoded, vec![0x08, 0x01, 0x00, 0x01, 0x30, 0x30, 0x00, 0x00]);

    let gcr2 = GCR::new();
    let reencoded = gcr2.encode(&decoded);
    assert_eq!(reencoded, encoded);
}
```

## Performance and allocation
- `encode` builds the output `Vec<u8>` by pushing 5 bytes per 4 input bytes; pre-sizing is not strictly necessary, but you can reserve capacity if you know the number of blocks.
- `decode` accumulates output and uses a small temporary for nibble packing; invalid codes short-circuit with `None`.

## Safety
This crate is `no_std`-unaware by default (it uses `Vec` from the standard library). It does not use unsafe code. There are no external dependencies.

## Testing
Run the tests:

```bash
cargo test
```

## Limitations and scope
- Only handles the 4-to-5 GCR mapping and 40-bit packing as implemented here.
- Does not include disk flux decoding/encoding, sync marks, sector layout, checksums, or higher-level track/sector handling.

## License
Licensed under either of
- Apache License, Version 2.0
- MIT license
at your option.

The declared license for this crate is "MIT OR Apache-2.0" as specified in Cargo.toml. If license text files are not present in the repository, refer to the standard license texts:
- Apache-2.0: https://www.apache.org/licenses/LICENSE-2.0
- MIT: https://opensource.org/licenses/MIT
