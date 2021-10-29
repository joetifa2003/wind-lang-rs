use crate::parser::Parser;
use clap::{App, Arg};
use interpreter::Interpreter;
// use pprof::protos::Message;
use scanner::Scanner;
use std::fs;
// use std::fs::File;
// use std::io::Write;

mod ast;
mod error;
mod interpreter;
mod parser;
mod scanner;
mod token;
mod types;

fn main() {
    // let guard = pprof::ProfilerGuard::new(997).unwrap();

    let matches = App::new("Wind Lang")
        .version("1.0")
        .author("Youssef Ahmed. <joetifa2003@gmail.com>")
        .about("A programming language implemented in Rust!")
        .arg(
            Arg::new("file")
                .long("file")
                .short('f')
                .value_name("FILE")
                .takes_value(true),
        )
        .get_matches();

    if let Some(file_name) = matches.value_of("file") {
        let code = fs::read_to_string(file_name).unwrap();

        let mut scanner = Scanner::new(code);
        let tokens = scanner.scan_tokens();

        let mut parser = Parser::new(tokens);
        let ast = parser.parse();

        let mut interpreter = Interpreter::new();
        interpreter.interpret(ast);
    }

    // if let Ok(report) = guard.report().build() {
    //     let mut file = File::create("profile.pb").unwrap();
    //     let profile = report.pprof().unwrap();

    //     let mut content = Vec::new();
    //     profile.encode(&mut content).unwrap();
    //     file.write_all(&content).unwrap();
    // };
}
