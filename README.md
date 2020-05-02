# Makefile Generator

## DISCLAIMER

This tool is not intended to be used for production level software or really large projects. It generates more like of a makefile template to work on (although the generated makefile works as is and will get the job done) with a pretty straightforward way of generation. That means, it doesn't try to make the most out of the sophisticated `make` tool. Long story short, it's a fast way of generating a makefile to get you up and running but it will probably not meet requirements for large projects, so don't use it that way :)

## About

**NOTE:** This program can generate only makefiles for C/C++ programs as it explicitly searches/handles the include structure of the C/C++ model (that means it won't work with C++20 modules).

This tool generates a makefile (the generated file is named **Makefile**) which can be automatically used to compile your project. What the program does is to read all the files with the designated extension (either c or cpp. For command line arguments please refer to **Command Line Arguments** section) and build the dependency graph for each file. That way it can generate the compilation of each C/C++ file to an object file with the right dependencies. Then all the object files are set as dependencies to the **bin** target which generates your binary (your executable). If you have other files with a `main` function, which will probably be your test files, you can filter them out and create a separate target named **tests** for these files (for more information check **Tests** section. More targets to come, such as examples).

## Getting started/Installation

In order to get the generator you must have Rust and cargo installed. These can easily be installed by executing the following command on terminal: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
For more information please refer [here](https://www.rust-lang.org/tools/install).

After you have installed Rust, you will have the cargo tool installed as well.

Now you can either execute `cargo install makegen` and get done with it or clone this repository and manually build the executable. In order to build it you must be inside the project's directory and run `cargo build --release` but this wont include the executable in your path, so you won't have it available from every directory in you system. You can easily have it available by creating a symlink in your binaries directory which is in you path by default. In order to achieve that, execute the following while being the project's directory:
`sudo ln -s $(pwd)/target/release/makegen /usr/bin/makegen`

## Generating a Makefile

In order to generate a makefile you must provide some arguments to the makegen executable. The mandatory parameters are `--binary` or `-b` for short and `--extension` or `-e` for short.
The first specifies the name of the binary that will be produced when compiling with `make`.
The second tells the tool to search for files with that extension (which can either be `c` r `cpp` for C and C++ files respectively. Please note that you don't need to prepend the dot `(.)` in to the extension argument).

**NOTE**: Please make sure that when running `makegen` you are in the root directory of the project you are creating the makefile for. 

So for example let's say I have a C++ project and a I want to generate a binary named `foo`.
In order to do that I must run: `makegen --binary=foo --extension=cpp`
This will generate a file named `Makefile` in the root of your project.

`makegen` supports other parameters which are explained below.
You can always run `makegen -h` or `makegen --help`  for a little more information.

## Choosing the Compiler

`makegen` sets the corresponding GNU compiler based on the `extension` parameter you provided by default.
That would be `gcc` for C files and `g++` for C++ files.
`makegen` gives you the option to choose the compiler by providing the `--compiler` or `-c` for short option.
Please note that `makegen` does not make a check of sorts to verify that the given compiler can compile C or C++ or that it is even a compiler.

## Optimization Level

By default `makegen` sets the optimization flag as `-O0` by default. If you want to override that you can provide the `--opt` flag. For example `makegen --binary=foo --extension=cpp --opt=O3`

## Choosing the Standard

By default `makegen` sets the compiler to use `-std=c99` if you are compiling C code or `-std=c++11` if you are compiling C++ code. You can override that by providing the `--std` flag.
For example `makegen --binary=foo --extension=cpp --std=c++17`

## Tests

`makegen` doesn't inspect each file to check if it has a `main` function. That means if you have multiple files
(more than one to be exact) which have a `main` function in them then the code will compile but the linker is going to be angry at you. That is going to happen because `makegen` will throw all generated object files to be linked for the `bin` target. The most common reason to have more than one file with a `main` function in it, is to have test cases or tests in general. So `makegen` provides a flag, named `--tests` or `-t` for short, in which you can specify test files or directories with test files and it will create a separate `tests` target for you. This flag defaults to the `tests` folder (it is ok if you don't have such a folder). So lets say we have a folder named `test_suite` which contains our test cases. In that case you must execute: `makegen --binary=foo --extension=cpp --tests=test_suite`
This will create a `tests` target in the Makefile, so you can execute `make tests` and compile only your tests.
`--tests` flag can appear multiple times and specify multiple folders, files that are tests.
For example let's say in the previous example we have another file named `test_foo.cpp`
In that case we would execute `makegen --binary=foo --extension=cpp --tests=test_suite --tests=test_foo.cpp`

## More to come

I would also like to make aware the tool about more common file categories that have a `main` function in them such as `examples` or `benchmarks`.
For any ideas please raise an Issue or contact me directly to georgeliontos98@gmail.com