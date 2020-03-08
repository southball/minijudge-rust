# MiniJudge-Rust

This is a judge process written in Rust.

## Features

- Executes all program in the [ioi/isolate](https://github.com/ioi/isolate/) sandbox.
- Supports Codeforces-style checker with `testlib.h`.
- Supports multiple judge processes running simultaneously.
- Supports outputting the final verdict to file with multiple format support.
- Supports C++17 and Python 3 at the moment (will support adding languages later)
- Has [ZeroMQ](https://zeromq.org/) publisher support to notify other software of judge progress

## Setting up

1. Clone the [ioi/isolate](https://github.com/ioi/isolate) directory and configure the `default.cf` file in the repository to pin each sandbox to the specific cores.
2. Install the sandbox with `sudo make install`.
3. Build the judge process with `cargo build --release`.
4. Run the judge process with `sudo minijudge-rust`. See the [Help](#Help) section below to see the required and optional flags.

## Metadata format

Apart from console-line flags, you need a file specifying the metadata of the problem. An example is given below.

- `problem_name`: the name of the problem.
- `time_limit`: the time limit (in seconds) for the submission to run (each test case).
- `memory_limit`: the memory limit (in KB) for the submission to run.
- `compile_time_limit`, `compile_memory_limit`, `checker_time_limit`, `checker_memory_limit`: the time limit (in seconds) and memory limit (in KB) for the compiler and the checker.
- `testcases`: the list of testcases for the program to be judged against. Each testcase must have a `input` and `output` field. Note that the path is relative to the `--testcases` flag provided in the judge program.

```yaml
problem_name: "A + B Problem"
time_limit: 1.0
memory_limit: 256000
compile_time_limit: 15.0
compile_memory_limit: 512000
checker_time_limit: 1.0
checker_memory_limit: 256000
testcases:
  - input: "1.in"
    output: "1.out"
  - input: "2.in"
    output: "2.out"
  - input: "3.in"
    output: "3.out"
  - input: "4.in"
    output: "4.out"
  - input: "5.in"
    output: "5.out"
```

## Help

```
minijudge-rust 0.0-alpha.1
Southball
MiniJudge-Rust A miniature judge written in Rust

USAGE:
    minijudge-rust [FLAGS] [OPTIONS] --metadata <metadata> --language <language> --source <source> --checker <checker> --testcases <testcases> --testlib <testlib> --sandboxes <sandboxes>

FLAGS:
    -h, --help       Prints help information
    -q, --quiet      Whether the log should be suppressed. This option overrides the verbose option
    -v, --verbose    The level of verbosity
    -V, --version    Prints version information

OPTIONS:
        --checker <checker>                  The path to the source code of checker. The source code must be written in
                                             C++
        --language <language>                The language that the source code was written in
        --metadata <metadata>                The path to a YAML file containing the metadata, including time limit,
                                             memory limit, test counts, etc
        --sandboxes <sandboxes>              The number of sandboxes to be created. The sandbox ID is 0-based
        --socket <socket>                    Socket to announce updates to. Events are emitted when test cases are
                                             completed, and when the whole submission is judged [default: ]
        --source <source>                    The path to the file containing source code
        --testcases <testcases>              The path to be used as the base path of the test cases files
        --testlib <testlib>                  The path to testlib.h
        --verdict <verdict>                  The file to output the verdict to [default: ]
        --verdict-format <verdict-format>    The format of the verdict to output [default: json]
```

