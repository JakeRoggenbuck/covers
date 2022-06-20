use std::fmt;
use std::fs;

struct Source {
    functions: Vec<String>,
    tests: Vec<String>,
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

fn isolate_functions_and_tests(contents: &str, source_ref: &mut Source) {
    let mut prev_line: &str = "";
    for mut line in contents.split("\n") {
        // The next line of code checks for fn, but also contains fn
        // But it also contains "exactly this" so it won't be caught
        // by the search for functions
        if line.contains("fn ") && line.contains("(") && !line.contains("exactly this") {
            line = line.trim_start();
            let parts: Vec<&str> = line.split("(").collect();
            line = parts[0];

            if prev_line.contains("#[test]") {
                source_ref.tests.push(line.to_string());
            } else {
                source_ref.functions.push(line.to_string());
            }
        }
        prev_line = line;
    }
}

fn read_tests_and_functions(path: &str, source_ref: &mut Source) {
    let contents = fs::read_to_string(path).expect("Could not open file");
    isolate_functions_and_tests(&contents, source_ref);
}

fn walk() -> Source {
    let mut source = Source {
        functions: Vec::<String>::new(),
        tests: Vec::<String>::new(),
    };

    let paths = fs::read_dir("./").unwrap();

    for path in paths {
        let new_path = path.unwrap().path();
        let path_name = new_path.to_string_lossy();

        if path_name.contains(".rs") {
            read_tests_and_functions(&path_name, &mut source);
        }
    }

    return source;
}

fn main() {
    let source = walk();
    println!("{}", source);
}

fn _function_to_test() -> i64 {
    return 0;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn _function_to_test_test() {
        assert!(_function_to_test() == 0);
    }
}
