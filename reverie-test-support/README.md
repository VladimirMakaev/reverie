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

## Injected-exit commands

The `exit` target keeps the original ptrace function guest and adds a command
that issues `getpid` followed by `exit_group(0)`. The actual shared
`InjectExitTool` replaces `getpid` with `exit_group(42)`. Both ptrace and the
optional authenticated DBT caller must return exit42 and exactly one exit42
process-hook RPC in the real returned GlobalState. Under DBT the RPC carries
the admitted terminal request; the child wait status separately checks the
actual exit. The original function guest is still ptrace-only.

The command fixture embeds its assembly and writes it into a test-owned
TempDir, so it needs no Cargo-specific runtime paths. Running these command
cases requires an x86-64 Linux C toolchain selected by `CC` or `cc`, supporting
`-nostdlib -static -no-pie -Wl,-e,_start -Wl,--build-id=none`. Missing compiler
capability fails the test. Every compiled image first runs natively and must
exit0 with empty streams; the TempDir lives through the native and traced run.

```sh
reverie-dbt/scripts/build-client.sh \
  --caller-manifest tests/fixtures/basics-noop-dbt-runtime/Cargo.toml \
  --profile debug --jobs 4 --check
reverie-dbt/scripts/build-client.sh \
  --caller-manifest tests/fixtures/exit-dbt-runtime/Cargo.toml \
  --profile debug --jobs 4 --check
cargo test -p reverie-test-support --features dbt --test exit -- \
  --test-threads=1
```

The default target has two tests; `dbt` adds the actual caller positive, a real
Noop negative and a missing-manifest guard. The Noop guest must first succeed
with exit0, empty streams and no application RPCs, then fail the same exit42
oracle. That control tests the first assertion's sensitivity; it does not
independently test each later RPC assertion. The missing-manifest guard first
authenticates the valid exit pair and does not run a guest. Caller initial-process
and signal-error restrictions remain. The helper installs `reverie=trace`
logging independently of `RUST_LOG`; these tests have no local timeout.

## Stat-family commands

The `stat` target preserves the two original ptrace function tests and adds
command forms using the same `LocalState` Tool. It subscribes to all syscalls
and uses the default identical tail forwarding; these tests do not establish
stat-to-fstatat conversion or a separate reinjection mechanism.

The first C guest checks that `open("/proc/self/exe", O_RDONLY)` returns a
positive descriptor, that `stat`, `lstat` and `fstat` succeed, and that the
followed inode equals the descriptor inode but differs from the symlink inode.
The second checks successful `fstatat` and raw `SYS_statx`, both with
`AT_SYMLINK_NOFOLLOW`, and equal returned inodes using `STATX_INO`.
These relations do not prove which executable `/proc/self/exe` names under
instrumentation.

The checked C sources are embedded and compiled in a test-owned TempDir using
`CC` or `cc`. They require an x86-64 Linux C toolchain and libc headers,
with assertions enabled. Dynamic loader and libc startup remain part of each
command run. Each compiled image must first run natively with exit0 and empty
streams, then satisfy the same guest assertions under ptrace or DBT with exit0
and empty streams. No inode values are compared across processes.

```sh
reverie-dbt/scripts/build-client.sh \
  --caller-manifest tests/fixtures/stat-dbt-runtime/Cargo.toml \
  --profile debug --jobs 4 --check
cargo test -p reverie-test-support --features dbt --test stat -- \
  --test-threads=1
```

The default target lists four tests. The optional `dbt` feature adds two
actual-caller positives and a missing-manifest guard that first authenticates
the real stat artifact. It does not provide function-guest execution through
DBT or extend the caller's lifecycle/signal support. The default forwarding
Tool is behaviorally equivalent to Noop for these assertions, so no wrong-Tool
sensitivity is claimed. The existing helper installs `reverie=trace` logging;
these tests have no local timeout.

## vDSO commands

The `vdso` target retains all four original ptrace function tests and shares
their actual `LocalState` with a command fixture. The fixture resolves the
versioned `LINUX_2.6` clock_gettime, gettimeofday, getcpu and clock_getres entries
from `linux-vdso.so.1`, and checks their mapping against `AT_SYSINFO_EHDR`.
Missing entries fail without substituting libc calls or raw syscalls.

The Tool returns EINVAL for clock_gettime, changes gettimeofday's microsecond
suffix to 345 after injection, writes CPU0, and supplies resolution `(0,42)`.
The direct-entry command checks those exact results. Direct vDSO errors use
negative errno, while the unchanged Rust Instant test still expects a panic.
Its broad `should_panic` attribute is not a precise error-cause assertion.
These tests do not compare timestamps or establish deterministic time.

The embedded C fixture requires an x86-64 Linux C toolchain selected by `CC`
or `cc`, libc headers and libdl, with assertions enabled. Its owned TempDir
lives through both runs. Each compiled image first runs natively, checking
successful calls and valid output ranges. The same image then runs through
ptrace or authenticated DBT with the Tool-specific expectations. Each mode
requires exit0 and empty streams. Native and traced values are not compared
across processes; dynamic startup remains part of each run.

```sh
reverie-dbt/scripts/build-client.sh \
  --caller-manifest tests/fixtures/vdso-dbt-runtime/Cargo.toml \
  --profile debug --jobs 4 --check
cargo test -p reverie-test-support --features dbt --test vdso -- \
  --test-threads=1
```

The default target has five tests. The optional `dbt` feature adds the actual
caller command and a missing-manifest guard, which authenticates the valid
pair before checking refusal. Unit global state alone does not authenticate
the selected Tool. No wrong-Tool sensitivity, DBT function-guest execution,
or expanded lifecycle/signal support is claimed. The helper installs
`reverie=trace` independently of `RUST_LOG`; these tests have no local timeout.
