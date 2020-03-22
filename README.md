# MiniJudge-Rust

This is a judge process written in Rust.

## Features

- Executes all program in the [ioi/isolate](https://github.com/ioi/isolate/) sandbox.
- Supports Codeforces-style checker with `testlib.h`.
- Supports multiple judge processes running simultaneously.
- Supports outputting the final verdict to file with multiple format support.
- Supports adding language through `languages.yml`.
- Has [ZeroMQ](https://zeromq.org/) publisher support to notify other software of judge progress

## Setting up

1. Clone the [ioi/isolate](https://github.com/ioi/isolate) directory and configure the `default.cf` file in the repository to pin each sandbox to the specific cores.
2. Install the sandbox with `sudo make install`.
3. Build the judge process with `cargo build --release`.
4. Run the judge process with `sudo minijudge-rust`. See the [Help](#Help) section below to see the required and optional flags.

## Languages setting

The path to a YAML file containing the definition to the languages should be passed to the judge process.

*Note that a C++17 (`cpp17`) entry is **always** required for compiling the checker.*

The list of fields are listed below:

- `code` (`string`, required): the language code passed to the judge for this language.
- `source_filename` (`string`, required): the filename to be used inside the sandbox for the source file.
- `executable_filename` (`string`, required): the filename to be used inside the sandbox for the executable file.
- `compile_command` (`string[]`, required): the list of **tokens** for the compile command. The tokens are formatted using Handlebars and two variables, `{{source}}` and `{{destination}}`, are passed to the template engine.
- `execute_command` (`string[]`, required): the list of **tokens** for the execute command. The tokens are formatted using Handlebars and one variable, `{{executable}}`, is passed to the template engine.
- `compile_flags` (`string[]`, optional): the list of additional flags to pass to the `ioi/isolate` sandbox when the executable is compiled from the source file.
- `execute_flags` (`string[]`, optional): the list of additional flags to pass to the `ioi/isolate` sandbox when the executable is executed.

Some sample entries for C++17, Python 3 and NodeJS are listed below:

```yaml
- code: "cpp17"
  source_filename: "source.cpp"
  executable_filename: "program"
  compile_command:
    - "/usr/bin/g++"
    - "--std=c++17"
    - "-O2"
    - "-o"
    - "{{destination}}"
    - "{{source}}"
  execute_command:
    - "{{executable}}"
- code: "python3"
  source_filename: "source.py"
  executable_filename: "program.py"
  compile_command:
    - "/bin/cp"
    - "{{source}}"
    - "{{destination}}"
  execute_command:
    - "/usr/bin/python3"
    - "{{executable}}"
- code: "nodejs"
  source_filename: "source.js"
  executable_filename: "program.js"
  compile_command:
    - "/bin/cp"
    - "{{source}}"
    - "{{destination}}"
  execute_command:
    - "/usr/bin/node"
    - "{{executable}}"
  execute_flags:
    - "--processes=0"
```

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
    minijudge-rust [FLAGS] [OPTIONS] --metadata <metadata> --language <language> --source <source> --checker <checker> --testcases <testcases> -
-testlib <testlib> --sandboxes <sandboxes> --languages-definition <languages-definition>

FLAGS:
    -h, --help       Prints help information
    -q, --quiet      Whether the log should be suppressed. This option overrides the verbose option
    -v, --verbose    The level of verbosity
    -V, --version    Prints version information

OPTIONS:
        --checker <checker>
            The path to the source code of checker. The source code must be written in C++

        --language <language>                            The language that the source code was written in
        --languages-definition <languages-definition>    The YAML file containing definition to different languages
        --metadata <metadata>
            The path to a YAML file containing the metadata, including time limit, memory limit, test counts, etc

        --sandboxes <sandboxes>
            The number of sandboxes to be created. The sandbox ID is 0-based

        --socket <socket>
            Socket to announce updates to. Events are emitted when test cases are completed, and when the whole
            submission is judged
        --source <source>                                The path to the file containing source code
        --testcases <testcases>                          The path to be used as the base path of the test cases files
        --testlib <testlib>                              The path to testlib.h
        --verdict <verdict>                              The file to output the verdict to
        --verdict-format <verdict-format>                The format of the verdict to output [default: json]
```
