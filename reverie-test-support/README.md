# Generic command tests

This package registers the existing `tests/basics.rs` source. Its command
helpers delegate to ptrace's existing runtime, output capture and caller-owned
Tool configuration. The Tools live once in `tests/tool_fixtures/basics.rs`.
Inherited function tests still use the original ptrace helpers.
The test dependency enables ptrace's existing `nightly` output-capture reset,
so inherited `println!` output reaches the guest pipe under ordinary libtest
capture. The workspace's pinned nightly toolchain supplies that API.

Run the complete registered target with ordinary libtest capture:

```sh
cargo test -p reverie-test-support --test basics
```

The initial command selection is:

```sh
cargo test -p reverie-test-support --test basics -- \
  --exact noop_tool_test counter_tool_test error_exit_test \
  run_guest_command_test run_guest_command_test_closure
```

The target also discovers the other existing basics declarations; running the
five command tests does not establish those other results. Sanitizer exclusions
remain unchanged.

The optional `dbt` feature adds five `dbt_` command identities, sharing all
thirteen original command assertions. The original thirteen basics tests and
the default command helpers keep using ptrace. The eight inherited function
tests do not run through DBT.

Produce the two concrete caller artifacts before selecting DBT tests:

```sh
reverie-dbt/scripts/build-client.sh \
  --caller-manifest tests/fixtures/basics-noop-dbt-runtime/Cargo.toml \
  --profile debug --jobs 4 --check
reverie-dbt/scripts/build-client.sh \
  --caller-manifest tests/fixtures/basics-counter-dbt-runtime/Cargo.toml \
  --profile debug --jobs 4 --check
cargo test -p reverie-test-support --features dbt --test basics --test dbt_command -- \
  --test-threads=1
```

The producers must use the consumer's profile and target directory. The tests
select `reverie-dbt-callers/basics-{noop,counter}/manifest.json` beside that
profile's `deps` directory. Each thin runtime alias exports the actual shared
fixture Tool; CounterLocal's existing RPC handler updates the returned real
CounterGlobal. The explicit `test_dbt_cmd_with_config` helper preserves inherited
stdin, cwd and the application environment and captures stdout/stderr. Missing
artifacts and caller activation errors are returned without a fallback.

The `dbt_command` target contains three controls: inherited stdin plus the
unchanged null-input method's negative control; the original count oracle
against actual CounterLocal and actual Noop runtimes; and missing-manifest
refusal. The count control requires successful guest execution and rejects a
setup refusal as evidence. The caller protocol's initial-process domain still
applies; signal termination is a launch error, not supported signal-status
fidelity. These command cases do not establish raw-environment transparency,
function-guest execution, or arbitrary-program support.
