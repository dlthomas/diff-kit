use std::io::prelude::*;
use unidiff::PatchSet;

fn main() {
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input).unwrap();
    let mut patches = PatchSet::new();
    patches.parse(input).ok().expect("error parsing diff");

    for file in patches {
        let target_file = &file.target_file.clone()[2..];
        for hunk in file {
            for line in hunk {
                if line.is_added() {
                    println!(
                        "{}:{}: {}",
                        target_file,
                        line.target_line_no.unwrap_or(0),
                        line.value
                    );
                }
            }
        }
    }
}
