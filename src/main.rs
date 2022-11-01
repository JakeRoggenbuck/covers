use colored::Colorize;
use std::fmt;
use std::fs;
use structopt::StructOpt;

struct Source {
    functions: Vec<String>,
    tests: Vec<String>,
    unwraps: i32,
}

struct Settings {
    only_with_returns: bool,
    full_line: bool,
}

#[derive(Debug, StructOpt)]
#[structopt(name = "Covers", about = "Check for test coverage and unwraps.")]
struct Opt {
    #[structopt(short, long)]
    only_with_returns: bool,

    #[structopt(short, long)]
    full_line: bool,
}

/// Display all of the functions and tests
impl fmt::Display for Source {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut out = String::from("Source:\n");
        out += "  Functions:\n";
        for func in &self.functions {
            out += &format!("    {},\n", func.blue()).to_string();
        }
        out += "  Tests:\n";
        for test in &self.tests {
            out += &format!("    {},\n", test.purple()).to_string();
        }

        write!(f, "{}", out)
    }
}

fn isolate_functions_and_tests(contents: &str, source_ref: &mut Source, settings: &Settings) {
    // Count and save the number of unwraps
    source_ref.unwraps = contents.matches(concat!("unwrap", "()")).count() as i32;

    let mut prev_line: &str = "";
    for line in contents.split("\n") {
        // The next line of code checks for fn, but also contains fn
        // But it also contains "exactly this" so it won't be caught
        // by the search for functions when checking covers on this file
        if line.contains("fn ") && line.contains("(") && !line.contains("exactly this") {
            let mut new_line;
            if !settings.full_line {
                // Remove things after '('
                new_line = line.trim_start();
                let parts: Vec<&str> = new_line.split("(").collect();
                new_line = parts[0];
            } else {
                new_line = line;
            }

            // Check that line starts with fn
            let start = &new_line[0..2];
            if start != "fn" {
                continue;
            }

            if prev_line.contains("#[test]") {
                source_ref.tests.push(new_line.to_string());
            } else {
                // wait for -> check as to not skip tests
                // skip if there is no return type
                if settings.only_with_returns {
                    if !line.contains("->") {
                        continue;
                    }
                }
                source_ref.functions.push(new_line.to_string());
            }
        }
        prev_line = line;
    }
}

fn read_tests_and_functions(path: &str, source_ref: &mut Source, settings: &Settings) {
    // This function does two things
    // It starts two things, one is the counting funcs and tests
    // The other is very basic, just counting unwraps
    let contents = fs::read_to_string(path).expect("Could not open file");
    isolate_functions_and_tests(&contents, source_ref, settings);
}

fn walk(settings: &Settings) -> Source {
    let mut source = Source {
        functions: Vec::<String>::new(),
        tests: Vec::<String>::new(),
        unwraps: 0,
    };

    let paths = match fs::read_dir("./") {
        Ok(a) => a,
        Err(e) => {
            println!("Count not read directory with error {e}");
            std::process::exit(1);
        }
    };

    // This assumes that tests are in the same file as the definition
    // not necessarily a safe but having the two in the same file is recommended
    for path in paths {
        let new_path = path.expect("Each path in paths should have path").path();
        let path_name = new_path.to_string_lossy();

        if path_name.contains(".rs") {
            read_tests_and_functions(&path_name, &mut source, settings);
        }
    }

    return source;
}

fn calc_final_score(unwraps: i32, cover_percent: f64) -> i32 {
    let mut score = 0;
    score += unwraps.pow(2);

    score += (100.0 - (cover_percent * 100.0)) as i32;
    return score;
}

fn show_test_cover(source: Source) {
    let tests_count = source.tests.len();
    let funcs_count = source.functions.len();

    let mut found = Vec::<usize>::new();

    for test in source.tests {
        for (i, func) in source.functions.iter().enumerate() {
            if test.contains(func) {
                println!("{} -> {}", func, test.green());
                found.push(i);
            }
        }
    }

    for (i, func) in source.functions.iter().enumerate() {
        if !found.contains(&i) {
            println!("{} -> {}", func, "X".red());
        }
    }

    let percent: f64 = (tests_count as f64 / funcs_count as f64) as f64;
    println!("\n=======================================================");
    println!(
        "Covers:   {:.4}% - {}/{}",
        percent, tests_count, funcs_count
    );
    println!("Unwraps:  {}", source.unwraps);
    println!("\nScore:    {}", calc_final_score(source.unwraps, percent));
    println!("closer to zero is better.");
}

fn main() {
    let opt = Opt::from_args();
    let settings = Settings {
        only_with_returns: opt.only_with_returns,
        full_line: opt.full_line,
    };

    let source = walk(&settings);
    println!("{source}");
    show_test_cover(source);
}

fn _function_to_test() -> i64 {
    return 0;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::path::Path;

    #[test]
    fn _function_to_test_test() {
        assert!(_function_to_test() == 0);
    }

    #[test]
    fn walk_test() {
        let settings = Settings {
            only_with_returns: false,
            full_line: false,
        };

        let source = walk(&settings);
        assert_eq!(source.functions.len(), 0);

        env::set_current_dir(Path::new("./src/"))
            .expect("Path ./src/ should exist and setting current directory should work.");

        let source = walk(&settings);
        assert_eq!(source.functions.len(), 9);
    }

    #[test]
    fn isolate_functions_and_tests_test() {
        const TEST_CODE: &str = "\
        fn this_is_some_code() { return 0.unwrap(); }
        \
        #[test]
        fn this_is_some_code_test() {
            assert!(this_is_some_code() == 0);
        }";

        let mut source = Source {
            functions: Vec::<String>::new(),
            tests: Vec::<String>::new(),
            unwraps: 0,
        };

        isolate_functions_and_tests(
            &TEST_CODE.to_string(),
            &mut source,
            &Settings {
                only_with_returns: false,
                full_line: false,
            },
        );
        assert_eq!(source.functions.len(), 1);
        assert_eq!(source.tests.len(), 1);

        assert_eq!(source.unwraps, 1);

        assert_eq!(source.functions, vec!["fn this_is_some_code"]);
        assert_eq!(source.tests, vec!["fn this_is_some_code_test"]);
    }

    #[test]
    fn calc_final_score_test() {
        assert_eq!(calc_final_score(0, 0.0), 100);
        assert_eq!(calc_final_score(0, 1.0), 0);
        assert_eq!(calc_final_score(0, 0.5), 50);

        assert_eq!(calc_final_score(2, 0.0), 104);
        assert_eq!(calc_final_score(4, 1.0), 16);
        assert_eq!(calc_final_score(9, 0.5), 131);
    }
}
