use colored::Colorize;
use std::fmt;
use std::fs;
use structopt::StructOpt;

struct Source {
    functions: Vec<String>,
    tests: Vec<String>,
}

struct Settings {
    only_with_returns: bool,
}

#[derive(Debug, StructOpt)]
#[structopt(name = "example", about = "An example of StructOpt usage.")]
struct Opt {
    #[structopt(short, long)]
    only_with_returns: bool,
}

impl fmt::Display for Source {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut out = String::from("Source:\n");
        out += "  Functions:\n";
        for func in &self.functions {
            out += &format!("    {func},\n").to_string();
        }
        out += "  Tests:\n";
        for test in &self.tests {
            out += &format!("    {test},\n").to_string();
        }

        write!(f, "{}", out)
    }
}

fn isolate_functions_and_tests(contents: &str, source_ref: &mut Source, settings: &Settings) {
    let mut prev_line: &str = "";
    for line in contents.split("\n") {
        // The next line of code checks for fn, but also contains fn
        // But it also contains "exactly this" so it won't be caught
        // by the search for functions
        if line.contains("fn ") && line.contains("(") && !line.contains("exactly this") {
            // Remove things after '('
            let mut new_line = line.trim_start();
            let parts: Vec<&str> = new_line.split("(").collect();
            new_line = parts[0];

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
    let contents = fs::read_to_string(path).expect("Could not open file");
    isolate_functions_and_tests(&contents, source_ref, settings);
}

fn walk(settings: &Settings) -> Source {
    let mut source = Source {
        functions: Vec::<String>::new(),
        tests: Vec::<String>::new(),
    };

    let paths = fs::read_dir("./").unwrap();

    for path in paths {
        let new_path = path.unwrap().path();
        let path_name = new_path.to_string_lossy();

        if path_name.contains(".rs") {
            read_tests_and_functions(&path_name, &mut source, settings);
        }
    }

    return source;
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
    println!("Covers {:.4}% - {}/{}", percent, tests_count, funcs_count);
}

fn main() {
    let opt = Opt::from_args();
    let settings = Settings {
        only_with_returns: opt.only_with_returns,
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
        };

        let source = walk(&settings);
        assert_eq!(source.functions.len(), 0);

        env::set_current_dir(Path::new("./src/")).unwrap();

        let source = walk(&settings);
        assert_eq!(source.functions.len(), 8);
    }

    #[test]
    fn isolate_functions_and_tests_test() {
        const TEST_CODE: &str = "\
        fn this_is_some_code() { return 0; }
        \
        #[test]
        fn this_is_some_code_test() {
            assert!(this_is_some_code() == 0);
        }";

        let mut source = Source {
            functions: Vec::<String>::new(),
            tests: Vec::<String>::new(),
        };

        isolate_functions_and_tests(
            &TEST_CODE.to_string(),
            &mut source,
            &Settings {
                only_with_returns: false,
            },
        );
        assert_eq!(source.functions.len(), 1);
        assert_eq!(source.tests.len(), 1);

        assert_eq!(source.functions, vec!["fn this_is_some_code"]);
        assert_eq!(source.tests, vec!["fn this_is_some_code_test"]);
    }
}
