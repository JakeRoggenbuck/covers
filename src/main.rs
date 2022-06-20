use std::fs;

struct Source {
    functions: Vec::<String>,
    tests: Vec::<String>,
}

fn read_tests_and_functions(path: &str) {
    let source = Source {
        functions: Vec::<String>::new(),
        tests: Vec::<String>::new(),
    };


}

fn walk() {
    let paths = fs::read_dir("./").unwrap();

    for path in paths {
        let new_path = path.unwrap().path();
        let path_name = new_path.to_string_lossy();

        if path_name.contains(".rs") {
            read_tests_and_functions(&path_name);
        }
    }
}

fn main() {
    walk();
}
