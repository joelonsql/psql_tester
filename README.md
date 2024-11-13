# psql_tester

Test suite for PostgreSQL `psql` `\copy` command, covering different input methods and formats.

## Overview

This test suite verifies `psql` `\copy` command behavior across different:
- Input methods (command, script, terminal)
- Data sources (file, stdin, tty)
- Data formats (text, csv, binary)

## Test Matrix

| Method   | Source  | Format | Test Name                                    |
|----------|---------|--------|----------------------------------------------|
| command  | file    | text   | test_psql_command__copy_from_file___text__   |
| command  | file    | csv    | test_psql_command__copy_from_file___csv___   |
| command  | file    | binary | test_psql_command__copy_from_file___binary   |
| script   | stdin   | text   | test_psql_script___copy_from_stdin__text__   |
| script   | stdin   | csv    | test_psql_script___copy_from_stdin__csv___   |
| script   | stdin   | binary | test_psql_script___copy_from_stdin__binary   |
| terminal | tty     | text   | test_psql_terminal_copy_from_tty____text__   |
| terminal | tty     | csv    | test_psql_terminal_copy_from_tty____csv___   |
| terminal | tty     | binary | test_psql_terminal_copy_from_tty____binary   |
| terminal | stdin   | text   | test_psql_terminal_copy_from_stdin__text__   |
| terminal | stdin   | csv    | test_psql_terminal_copy_from_stdin__csv___   |
| terminal | stdin   | binary | test_psql_terminal_copy_from_stdin__binary   |

## Prerequisites

- Rust toolchain
- PostgreSQL client (psql)
- Running PostgreSQL server
- Environment variables:
  - `PGDATABASE` or default to current user
  - Standard PostgreSQL environment variables (if needed): `PGHOST`, `PGPORT`, `PGUSER`, etc.

## Running Tests

```sh
cargo test
```

```
   Compiling psql_tester v0.1.0 (/home/foo/src/psql_tester)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.48s
     Running unittests src/main.rs (target/debug/deps/psql_tester-763ec3e278ec9537)

running 12 tests
test tests::test_psql_command__copy_from_file___text__ ... ok
test tests::test_psql_command__copy_from_file___csv___ ... ok
test tests::test_psql_command__copy_from_file___binary ... ok
test tests::test_psql_script___copy_from_stdin__text__ ... ok
test tests::test_psql_script___copy_from_stdin__csv___ ... ok
test tests::test_psql_script___copy_from_stdin__binary ... ok
test tests::test_psql_terminal_copy_from_tty____text__ ... ok
test tests::test_psql_terminal_copy_from_tty____csv___ ... ok
test tests::test_psql_terminal_copy_from_tty____binary ... ok
test tests::test_psql_terminal_copy_from_stdin__text__ ... ok
test tests::test_psql_terminal_copy_from_stdin__csv___ ... ok
test tests::test_psql_terminal_copy_from_stdin__binary ... ok

test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.13s
```

## License

This project is licensed under the PostgreSQL License.
