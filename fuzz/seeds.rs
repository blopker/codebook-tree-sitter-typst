//! Writes every test input of the given tree-sitter corpus files into a
//! directory, one file per input, for use as a libFuzzer seed corpus.
//!
//! Usage: `seeds <out-dir> <corpus-file>...`

use std::path::Path;

/// A corpus file separates its sections with lines of `=` or `-`.
fn is_rule(line: &str, c: char) -> bool {
    line.len() >= 20 && line.chars().all(|x| x == c)
}

/// Returns the inputs of a corpus file: each follows a name enclosed by two
/// lines of `=` and runs up to a line of `-`.
fn inputs(corpus: &str) -> Vec<&str> {
    enum State {
        Tree,
        Name,
        Input(usize),
    }
    let mut found = Vec::new();
    let mut state = State::Tree;
    let mut position = 0;
    for line in corpus.split_inclusive('\n') {
        let next = position + line.len();
        let text = line.trim_end_matches('\n');
        state = match state {
            State::Input(start) if is_rule(text, '-') => {
                found.push(corpus[start..position].trim_end_matches('\n'));
                State::Tree
            }
            State::Tree if is_rule(text, '=') => State::Name,
            State::Name if is_rule(text, '=') => State::Input(next),
            other => other,
        };
        position = next;
    }
    found
}

fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("usage: {} <out-dir> <corpus-file>...", args[0]);
        std::process::exit(2);
    }
    let out_dir = Path::new(&args[1]);
    std::fs::create_dir_all(out_dir)?;
    let mut count = 0;
    for path in &args[2..] {
        let stem = Path::new(path).file_stem().unwrap().to_string_lossy();
        for (index, input) in inputs(&std::fs::read_to_string(path)?).iter().enumerate() {
            std::fs::write(out_dir.join(format!("{stem}-{index:04}.typ")), input)?;
            count += 1;
        }
    }
    println!("wrote {count} seeds to {}", out_dir.display());
    Ok(())
}
