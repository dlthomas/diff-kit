use std::{collections::BTreeSet, fs::read_to_string, io::prelude::*, path::Path, rc::Rc};

use unidiff::PatchSet;

fn main() {
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input).unwrap();
    let mut patches = PatchSet::new();
    patches.parse(input).ok().expect("error parsing diff");

    let mut executed = BTreeSet::new();
    for arg in std::env::args().skip(1) {
        let prefix = Path::new(&arg)
            .ancestors()
            .find(|p| p.join(".git").exists())
            .expect(".git not found in path to root from argument");

        match read_to_string(&arg) {
            Err(err) => eprintln!("error reading file {}: {}", arg, err),
            Ok(json) => match json::parse(&json) {
                Err(err) => eprintln!("error parsing json: {}", err),
                Ok(coverage) => {
                    for (file, coverage) in coverage.entries() {
                        let mut statements = BTreeSet::new();
                        let file = Path::new(file).strip_prefix(prefix).unwrap();
                        let file = Rc::new(file.to_string_lossy().to_string());
                        for (idx, count) in coverage["s"].entries() {
                            if let Some(count) = count.as_usize() {
                                if count > 0 {
                                    statements.insert(idx);
                                }
                            }
                        }

                        let map = &coverage["statementMap"];
                        for stmt in statements {
                            let obj = &map[stmt];
                            let start = obj["start"]["line"]
                                .as_usize()
                                .expect("unexpected value for start-of-statement line number");
                            let end = obj["end"]["line"]
                                .as_usize()
                                .expect("unexpected value for end-of-statement line number");

                            for line in start..end {
                                executed.insert((file.clone(), line));
                            }
                        }
                    }
                }
            },
        }
    }

    for file in patches {
        let target_file = Rc::new(file.target_file[2..].to_string());
        for hunk in file {
            let mut last_context = None;
            for line in hunk {
                if line.is_context() {
                    last_context = Some(line.clone())
                }

                if line.is_removed() {
                    let line_no = match &last_context {
                        None => 1,
                        Some(line) => line.target_line_no.unwrap_or(0),
                    };

                    if executed.contains(&(target_file.clone(), line_no))
                        || executed.contains(&(target_file.clone(), line_no + 1)) {
                        println!("{}:{}: - {}", target_file, line_no, line.value)
                    }
                }

                if line.is_added() {
                    if let Some(line_no) = line.target_line_no {
                        if executed.contains(&(target_file.clone(), line_no)) {
                            println!("{}:{}: + {}", target_file, line_no, line.value)
                        }
                    }
                }
            }
        }
    }
}
