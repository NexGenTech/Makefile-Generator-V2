use clap::ArgMatches;
use std::collections::HashSet;

pub struct Config<'conf> {
    pub compiler: &'conf str,
    pub extension: &'conf str,
    pub program_name: &'conf str,
    pub standard: &'conf str,
    pub opt_level: &'conf str,
    pub tests: HashSet<&'conf str>,
}

impl<'conf> Config<'conf> {
    pub fn from_matches(matches: &'conf ArgMatches<'conf>) -> Result<Self, &'static str> {
        let bin = matches
            .value_of("bin")
            .ok_or("You must provide a name for your executable")?;

        let extension = matches
            .value_of("extension")
            .ok_or("You must provide and file extension to search for")?;
        if extension != "c" && extension != "cpp" {
            return Err("Only C or C++ files are allowed (extension should be either c or cpp)");
        }

        let compiler = matches.value_of("compiler").ok_or("")?;

        let standard = matches.value_of("std").unwrap();

        let opt_level = matches.value_of("opt").unwrap();

        let tests = matches
            .values_of("tests")
            .unwrap()
            .collect::<HashSet<&'conf str>>();

        Ok(Self {
            compiler,
            extension,
            standard,
            opt_level,
            tests,
            program_name: bin,
        })
    }
}
