#[macro_use]
extern crate lazy_static;

mod config;
mod generator;
mod parser;

use clap::{App, Arg};
use config::Config;
use generator::MakefileGenerator;
use parser::Parser;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let matches = App::new("makegen")
        .version("1.0")
        .author("George Liontos <georgios.liontos@sgdigital.com>")
        .about("Generate C/C++ makefiles quickly and easily!")
        .arg(
            Arg::with_name("compiler")
                .short("c")
                .long("compiler")
                .value_name("COMPILER")
                .help("Choose what compiler to use when compiling")
                .default_value_if("extension", Some("c"), "gcc")
                .default_value_if("extension", Some("cpp"), "g++")
                .takes_value(true)
                .min_values(1)
                .max_values(1),
        )
        .arg(Arg::with_name("extension")
            .short("e")
            .long("extension")
            .value_name("EXTENSION")
            .help("Choose what extensions should the generator look for. It must be c for C files and cpp for C++ files")
            .takes_value(true)
            .min_values(1)
            .max_values(1)
            .required(true),
        )
        .arg(Arg::with_name("bin")
            .short("b")
            .long("binary")
            .value_name("PROGRAM_NAME")
            .help("Choose what the program of the generated executable should be")
            .takes_value(true)
            .min_values(1)
            .max_values(1)
            .required(true),
        )
        .arg(Arg::with_name("std")
            .long("std")
            .value_name("C/C++ Standard")
            .help("Specifies the standard to use when compiling")
            .takes_value(true)
            .default_value_if("extension", Some("c"), "c99")
            .default_value_if("extension", Some("cpp"), "c++11")
            .min_values(1)
            .max_values(1),
        )
        .arg(Arg::with_name("opt")
            .long("opt")
            .value_name("OPTIMIZATION_LEVEL")
            .help("Specifies the optimization level to include in the compiler flags")
            .takes_value(true)
            .default_value("O0")
            .min_values(1)
            .max_values(1),
        )
        .arg(Arg::with_name("tests")
            .short("t")
            .long("tests")
            .value_name("TEST_FILE|TEST_DIRECTORY")
            .help("Specifies the directory or files that are tests files and include a main function")
            .takes_value(true)
            .default_value("tests")
            .multiple(true)
            .min_values(1),
        )
        .get_matches();

    let config = Config::from_matches(&matches)?;
    let root_dir = std::env::current_dir()?;
    let parser = Parser::new(root_dir, &config);
    let result = parser.parse()?;
    let generator = MakefileGenerator::new(config, result);
    generator.generate_makefile()?;
    Ok(())
}
