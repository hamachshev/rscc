use rscc::lexer;
use std::{env::args, path::PathBuf};

#[allow(unused)]
fn main() {
    let path: PathBuf = args()
        .nth(1)
        .expect("Must input a c file to compile")
        .into();

    let file = std::fs::File::open(path).unwrap();
    println!("{:?}", lexer::lex(file))
}

#[cfg(test)]
mod test {
    use std::{fs, process::Command};

    #[test]
    fn test() {
        let valid_tests = fs::read_dir("write_a_c_compiler-master/stage_1/valid").unwrap();
        let invalid_tests = fs::read_dir("write_a_c_compiler-master/stage_1/invalid").unwrap();

        for test in valid_tests.chain(invalid_tests) {
            let test = test.unwrap();
            println!("{}", test.path().to_str().unwrap());
            let res = Command::new("./target/debug/rscc")
                .arg(test.path().to_str().unwrap())
                .output()
                .unwrap();
            println!("{}", String::from_utf8_lossy(&res.stdout))
        }
    }
}
