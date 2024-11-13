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
| command  | file    | text   | command_file::text::test_psql_copy          |
| command  | file    | csv    | command_file::csv::test_psql_copy           |
| command  | file    | binary | command_file::binary::test_psql_copy        |
| script   | stdin   | text   | script_stdin::text::test_psql_copy          |
| script   | stdin   | csv    | script_stdin::csv::test_psql_copy           |
| script   | stdin   | binary | script_stdin::binary::test_psql_copy        |
| terminal | tty     | text   | terminal_tty::text::test_psql_copy          |
| terminal | tty     | csv    | terminal_tty::csv::test_psql_copy           |
| terminal | tty     | binary | terminal_tty::binary::test_psql_copy        |
| terminal | stdin   | text   | terminal_stdin::text::test_psql_copy        |
| terminal | stdin   | csv    | terminal_stdin::csv::test_psql_copy         |
| terminal | stdin   | binary | terminal_stdin::binary::test_psql_copy      |

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
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.01s
     Running unittests src/main.rs (target/debug/deps/psql_tester-763ec3e278ec9537)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/common.rs (target/debug/deps/common-9b4c1db3889d8dd4)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/mod.rs (target/debug/deps/integration-d7b9e68654ced786)

running 12 tests
test terminal_tty::text::test_psql_copy ... ok
test terminal_tty::csv::test_psql_copy ... ok
test terminal_stdin::csv::test_psql_copy ... ok
test terminal_stdin::text::test_psql_copy ... ok
test script_stdin::text::test_psql_copy ... ok
test script_stdin::binary::test_psql_copy ... ok
test command_file::binary::test_psql_copy ... ok
test script_stdin::csv::test_psql_copy ... ok
test command_file::csv::test_psql_copy ... ok
test command_file::text::test_psql_copy ... ok
test terminal_stdin::binary::test_psql_copy ... ok
test terminal_tty::binary::test_psql_copy ... ok

test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.12s
```

## License

This project is licensed under the PostgreSQL License.
