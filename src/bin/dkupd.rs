use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use unidiff::PatchSet;

fn main() -> std::io::Result<()> {
    let args: Vec<_> = std::env::args().collect();
    let prefix = 1;
    let diff_file = match args.as_slice() {
        [_, diff_file] => diff_file,
        _ => panic!("expected exactly one argument"),
    };

    let strip_prefix = |filename: &String| {
        let mut components = std::path::Path::new(filename).components();
        for _ in 0..prefix {
            components.next();
        }
        String::from(components.as_path().to_str().expect("file path in diff contains non-UTF8"))
    };

    let patches = {
        let mut patches = std::collections::HashMap::new();
        let mut patch_set = PatchSet::new();
        let mut diff_file = File::open(diff_file)?;
        let mut diff = String::new();
        diff_file.read_to_string(&mut diff).expect("error reading diff");
        patch_set.parse(diff).ok().expect("error parsing diff");

        for patched_file in patch_set {
            let filename = strip_prefix(&patched_file.source_file);
            patches.insert(filename, patched_file);
        }

        patches
    };

    let pattern = regex::Regex::new("^([^:]+):([0-9]+):([0-9]:)? ").expect("failed to parse regex");

    let update_location = |captures: &regex::Captures<'_>| -> String {
        let filename = String::from(captures.get(1).unwrap().as_str());
        let lineno = str::parse::<usize>(captures.get(2).unwrap().as_str()).unwrap();
        // TODO: handle offset, when present
        let patched_file = match patches.get(&filename) {
            None => return String::from(format!("{}:{}: ", strip_prefix(&filename), lineno)),
            Some(patched_file) => patched_file,
        };

        let mut tgtlineno = lineno;
        'hunks: for hunk in patched_file.clone().into_iter() {
            if hunk.source_start > lineno {
                break
            }

            if hunk.source_start + hunk.source_length < lineno {
                tgtlineno += hunk.target_length - hunk.source_length;
            } else {
                for line in hunk.into_iter() {
                    if let Some(source_line_no) = line.source_line_no {
                        if source_line_no >= lineno {
                            break 'hunks
                        }
                    }

                    if line.is_added() {
                        tgtlineno += 1
                    }

                    if line.is_removed() {
                        tgtlineno -= 1
                    }
                }
            }
        }

        format!("{}:{}: ", filename, tgtlineno)
    };

    for line in BufReader::new(std::io::stdin()).lines() {
        let line = line?;
        let line = pattern.replace(&line, update_location);
        println!("{}", line);
    }

    Ok(())
}
