#![allow(unused_must_use)]
use crate::config::Config;
use crate::parser::ParseResult;
use std::{collections::HashSet, fs::File, io::prelude::*, path::Path};

fn has_extension<P: AsRef<Path>>(path: P, ext: &str) -> bool {
    path.as_ref()
        .extension()
        .map(|e| e.to_str().unwrap_or("") == ext)
        .unwrap_or(false)
}

fn strip_extesion(source: &str) -> &str {
    if let Some(ext_index) = source.find(".") {
        &source[..ext_index]
    } else {
        source
    }
}

pub struct MakefileGenerator<'conf> {
    config: Config<'conf>,
    parse_result: ParseResult,
}

impl<'conf> MakefileGenerator<'conf> {
    pub fn new(config: Config<'conf>, parse_result: ParseResult) -> Self {
        Self {
            config,
            parse_result,
        }
    }

    pub fn generate_makefile(&self) -> std::io::Result<()> {
        let mut makefile = File::create("Makefile")?;
        self.generate_compiler_variables(&mut makefile);
        self.generate_file_variables(&mut makefile);
        self.generate_targets(&mut makefile);
        Ok(())
    }

    fn generate_compiler_variables(&self, makefile: &mut File) {
        writeln!(makefile, "CC := {}", self.config.compiler);
        writeln!(makefile, "CFLAGS := -Wall");
        writeln!(makefile, "CFLAGS += -std={}", self.config.standard);
        writeln!(makefile, "CLFAGS += -{}", self.config.opt_level);
        writeln!(
            makefile,
            "LFLAGS := {}",
            self.parse_result
                .dlls
                .iter()
                .map(|dll| format!("-l{}", dll))
                .collect::<Vec<String>>()
                .join(" ")
        );
    }

    fn generate_file_variables(&self, makefile: &mut File) {
        writeln!(makefile);
        writeln!(makefile, "ODIR := .OBJ\n");
        for file in self
            .parse_result
            .dependency_map
            .keys()
            .filter(|f| has_extension(f, self.config.extension))
        {
            self.generate_dependencies_variable_for_file(file, makefile);
        }
    }

    fn generate_dependencies_variable_for_file(&self, file: &str, makefile: &mut File) {
        write!(makefile, "{} := ", self.get_deps_var_name_for_file(file));
        self.print_file_dependecies(&file, makefile);
        writeln!(makefile);
    }

    fn print_file_dependecies(&self, filename: &str, makefile: &mut File) {
        let mut seen = HashSet::new();
        self.print_file_dependecies_r(filename, &mut seen, makefile);
    }

    fn print_file_dependecies_r(
        &self,
        filename: &str,
        seen: &mut HashSet<String>,
        makefile: &mut File,
    ) {
        if self.parse_result.dependency_map.contains_key(filename) {
            write!(makefile, "{} ", filename);
            for dependency in self.parse_result.dependency_map.get(filename).unwrap() {
                if !seen.contains(dependency) {
                    seen.insert(dependency.to_owned());
                    self.print_file_dependecies_r(dependency, seen, makefile);
                }
            }
        }
    }

    fn generate_targets(&self, makefile: &mut File) {
        self.generate_object_dir_target(makefile);
        writeln!(makefile, "all: bin\n");

        self.generate_bin_target(makefile);
        self.generate_bin_files_targets(makefile);
        if self
            .parse_result
            .dependency_map
            .keys()
            .any(|k| self.config.tests.iter().any(|v| v == k || k.starts_with(v)))
        {
            self.generate_test_target(makefile);
            self.generate_test_file_targets(makefile);
        }
        self.parse_result
            .dependency_map
            .keys()
            .filter(|k| {
                self.config
                    .tests
                    .iter()
                    .any(|v| k.starts_with(*v) || k == v)
            })
            .collect::<Vec<_>>();
        self.generate_clean_target(makefile);
    }

    fn generate_object_dir_target(&self, makefile: &mut File) {
        writeln!(makefile, "\n$(ODIR):");
        writeln!(makefile, "\t@mkdir $(ODIR)\n");
    }

    fn generate_bin_target(&self, makefile: &mut File) {
        let bin_object_files = self
            .parse_result
            .dependency_map
            .keys()
            .filter(|f| has_extension(f, self.config.extension))
            .filter(|f| {
                self.config
                    .tests
                    .iter()
                    .any(|v| f != v && !f.starts_with(*v))
            })
            .map(|f| strip_extesion(f))
            .map(|f| format!("$(ODIR)/{}.o", f.replace("/", "_")))
            .collect::<Vec<String>>()
            .join(" ");
        writeln!(makefile, "BIN_OBJ_FILES := {}\n", bin_object_files);
        writeln!(makefile, "bin: $(ODIR) $(BIN_OBJ_FILES)");
        writeln!(
            makefile,
            "\t$(CC) $(BIN_OBJ_FILES) -o {} $(LFLAGS)\n",
            self.config.program_name
        );
    }

    fn generate_bin_files_targets(&self, makefile: &mut File) {
        for file in self
            .parse_result
            .dependency_map
            .keys()
            .filter(|f| has_extension(f, self.config.extension))
            .filter(|f| {
                self.config
                    .tests
                    .iter()
                    .any(|v| f != v && !f.starts_with(*v))
            })
            .map(|f| strip_extesion(f))
        {
            let escaped_file = file.replace("/", "_");
            write!(
                makefile,
                "$(ODIR)/{}.o: $({})",
                escaped_file,
                self.get_deps_var_name_for_file(&format!("{}.{}", file, self.config.extension))
            );
            writeln!(makefile);
            writeln!(
                makefile,
                "\t$(CC) -c {}.{} $(CFLAGS) -o $(ODIR)/{}.o\n",
                file, self.config.extension, escaped_file
            );
        }
    }

    fn generate_test_target(&self, makefile: &mut File) {
        let test_targets = self
            .parse_result
            .dependency_map
            .keys()
            .filter(|k| {
                self.config
                    .tests
                    .iter()
                    .any(|v| k.starts_with(*v) || k == v)
            })
            .map(|f| strip_extesion(f).replace("/", "_"))
            .collect::<Vec<_>>()
            .join(" ");

        writeln!(
            makefile,
            "ALL_BIN_OBJS_WO_MAIN := $(filter-out $(ODIR)/main.o, $(BIN_OBJ_FILES))\n"
        );
        writeln!(makefile, "tests: {}\n", test_targets);
    }

    fn generate_test_file_targets(&self, makefile: &mut File) {
        for test in self.parse_result.dependency_map.keys().filter(|k| {
            self.config
                .tests
                .iter()
                .any(|v| k.starts_with(*v) || k == v)
        }) {
            let target_name = strip_extesion(test).replace("/", "_");

            writeln!(
                makefile,
                "{} : $({})",
                target_name,
                self.get_deps_var_name_for_file(test)
            );
            writeln!(
                makefile,
                "\t$(CC) $(CFLAGS) {} $(ALL_BIN_OBJS_WO_MAIN) -o {}\n",
                test,
                strip_extesion(test)
            );
        }
    }

    fn generate_clean_target(&self, makefile: &mut File) {
        writeln!(makefile, ".PHONY: clean\n");
        writeln!(makefile, "clean:");
        writeln!(makefile, "\trm -rf .OBJ {}", self.config.program_name);
    }
    fn get_deps_var_name_for_file(&self, filename: &str) -> String {
        let var_name = &filename[..(filename.find(".").unwrap())];
        let var_name = var_name.replace("/", "_");
        format!("{}_DEPS", var_name.to_ascii_uppercase())
    }
}
