//! libFuzzer target that parses arbitrary bytes with the Typst grammar. Run it
//! with `make fuzz`, which compiles the grammar and the tree-sitter runtime
//! with sanitizers so that memory errors in the C code end the run.
#![no_main]

use std::hash::{DefaultHasher, Hash, Hasher};
use std::time::{Duration, Instant};

use libfuzzer_sys::fuzz_target;
use tree_sitter::{ParseOptions, ParseState, Parser};

/// A parse that takes longer than this is cancelled. The grammar has known
/// slow inputs, so they are saved for inspection instead of ending the run.
const SLOW: Duration = Duration::from_secs(5);

fuzz_target!(|data: &[u8]| {
    let mut parser = Parser::new();
    parser
        .set_language(&codebook_tree_sitter_typst::LANGUAGE.into())
        .expect("cannot load the Typst grammar");

    let start = Instant::now();
    let mut too_slow = |_: &ParseState| start.elapsed() >= SLOW;
    let tree = parser.parse_with_options(
        &mut |byte, _| data.get(byte..).unwrap_or_default(),
        None,
        Some(ParseOptions::new().progress_callback(&mut too_slow)),
    );

    if tree.is_none() {
        // TYPST_FUZZ_SLOW_DIR names the directory that collects slow inputs.
        if let Some(dir) = std::env::var_os("TYPST_FUZZ_SLOW_DIR") {
            let mut hasher = DefaultHasher::new();
            data.hash(&mut hasher);
            let path = std::path::Path::new(&dir).join(format!("slow-{:016x}", hasher.finish()));
            let _ = std::fs::write(path, data);
        }
    }
});
