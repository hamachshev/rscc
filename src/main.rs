use clap::Parser;
use rscc::{codegen, lexer, parser};
use std::{
    ffi::OsString,
    fs::DirBuilder,
    io::{self, Write},
    os::unix::ffi::OsStrExt,
    path::{Path, PathBuf},
};

#[allow(unused)]
fn main() {
    let args = Args::parse();
    let mut path: PathBuf = args.file;
    let output_filename = args.output_filename;
    let output_to_std_out = args.output_to_sdout == Some('-');

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
    if output_to_std_out {
        io::stdout()
            .write_all(codegen.as_bytes())
            .expect("couldnt write to stdout");
    } else {
        std::fs::write(&output_filename, codegen).expect(&format!(
            "couldnt write to file {}",
            &output_filename_string
        ));
        println!("compiled to {}", output_filename_string);
    }
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

///rscc - c compiler written in rust
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    file: PathBuf,
    /// output file path. default is <path-of-c-file>.s
    #[arg(short)]
    output_filename: Option<PathBuf>,

    /// Output to stdout instead of file
    output_to_sdout: Option<char>,
}
