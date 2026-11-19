//! Feeds the external scanner's deserialize function buffers that serialize
//! would never write. `make scanner-test` runs this under AddressSanitizer, so
//! a read past the buffer or a write past the scanner's own storage fails the
//! test.

use std::ffi::c_void;

unsafe extern "C" {
    fn tree_sitter_typst_external_scanner_create() -> *mut c_void;
    fn tree_sitter_typst_external_scanner_destroy(scanner: *mut c_void);
    fn tree_sitter_typst_external_scanner_serialize(scanner: *mut c_void, buffer: *mut u8) -> u32;
    fn tree_sitter_typst_external_scanner_deserialize(
        scanner: *mut c_void,
        buffer: *const u8,
        length: u32,
    );
}

/// Size of the buffer tree-sitter hands to serialize.
const BUFFER_SIZE: usize = 1024;

/// Deserializes `bytes` from an allocation of exactly that size, so the
/// sanitizer sees any read beyond them, and returns what the scanner
/// serializes afterwards.
fn roundtrip(bytes: &[u8]) -> Vec<u8> {
    // Force the grammar's object files, which hold the scanner, to be linked.
    let _ = codebook_tree_sitter_typst::LANGUAGE;
    let exact = bytes.to_vec().into_boxed_slice();
    let mut out = vec![0u8; BUFFER_SIZE];
    unsafe {
        let scanner = tree_sitter_typst_external_scanner_create();
        tree_sitter_typst_external_scanner_deserialize(scanner, exact.as_ptr(), exact.len() as u32);
        let written = tree_sitter_typst_external_scanner_serialize(scanner, out.as_mut_ptr());
        tree_sitter_typst_external_scanner_destroy(scanner);
        out.truncate(written as usize);
    }
    out
}

fn with_first_vector_length(state: &[u8], length: usize) -> Vec<u8> {
    let mut bytes = state.to_vec();
    bytes[..std::mem::size_of::<usize>()].copy_from_slice(&length.to_ne_bytes());
    bytes
}

#[test]
fn valid_state_survives_a_round_trip() {
    let fresh = roundtrip(&[]);
    assert!(!fresh.is_empty());
    assert_eq!(roundtrip(&fresh), fresh);
}

#[test]
fn truncated_state_resets_the_scanner() {
    let fresh = roundtrip(&[]);
    for length in 1..fresh.len() {
        assert_eq!(roundtrip(&fresh[..length]), fresh, "length {}", length);
    }
}

#[test]
fn vector_length_past_the_end_resets_the_scanner() {
    let fresh = roundtrip(&[]);
    assert_eq!(roundtrip(&with_first_vector_length(&fresh, 300)), fresh);
    assert_eq!(roundtrip(&with_first_vector_length(&fresh, usize::MAX / 2)), fresh);
}

#[test]
fn trailing_bytes_reset_the_scanner() {
    let fresh = roundtrip(&[]);
    let mut bytes = fresh.clone();
    bytes.push(0);
    assert_eq!(roundtrip(&bytes), fresh);
}
