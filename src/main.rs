use rscc::{codegen, lexer, parser};
use std::{env::args, ffi::OsString, fs::DirBuilder, os::unix::ffi::OsStrExt, path::PathBuf};

#[allow(unused)]
fn main() {
    let mut args = args();
    let mut path: PathBuf = args.nth(1).expect("Must input a c file to compile").into();
    let output_filename = match args.next() {
        Some(arg) if arg == "-o" => Some(args.next().expect("Expected filename after -o")),
        Some(_) => panic!("Unexpected argument. Usage <input.c> [-o output]"),
        None => None,
    };
    let file = std::fs::File::open(&path).unwrap();
    let lex = lexer::lex(file).unwrap();
    let parse = parser::parse_program(&mut lex.into_iter().peekable());
    let codegen = codegen::gen_program(parse);
    let output_filename = match output_filename {
        Some(n) => OsString::from(n),
        None => {
            let c_path = path;
            let mut path = PathBuf::from("build/");
            if !path.exists() {
                DirBuilder::new()
                    .create(&path)
                    .expect("failed to create build dir")
            }
            path.push(c_path);
            path.set_extension("s");
            OsString::from(path)
        }
    };
    let output_filename_string = String::from_utf8_lossy(output_filename.as_bytes());
    std::fs::write(&output_filename, codegen).expect(&format!(
        "couldnt write to file {}",
        &output_filename_string
    ));
    println!("compiled to {}", output_filename_string)
}

#[cfg(test)]
mod test {
    use std::{fs, process::Command};

    #[test]
    fn test() {
        let valid_tests = fs::read_dir("write_a_c_compiler-master/stage_2/valid").unwrap();
        let invalid_tests = fs::read_dir("write_a_c_compiler-master/stage_2/invalid").unwrap();

        for test in valid_tests.chain(invalid_tests) {
            let test = test.unwrap();
            println!("{}", test.path().to_str().unwrap());
            let res = Command::new("./target/debug/rscc")
                .arg(test.path().to_str().unwrap())
                .output()
                .unwrap();
            println!("{}", String::from_utf8_lossy(&res.stdout));
            println!("{}", &res.status);
        }
    }
}
