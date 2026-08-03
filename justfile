alias b := build
alias c := clean
alias d := debug
alias t := test

set shell := ["zsh", "-uc"]

[doc("Choose one available receip")]
default:
    @just --choose

[arg("mode", long="release", short="r", value="release", help="set cargo build mode to release")]
[doc("Configure and build the project")]
[group("run")]
build mode="debug":
    cargo build {{ if mode == "release" { "--release" } else { "" } }}

[arg("mode", long="release", short="r", value="release", help="set cargo build mode to release")]
[doc("Running an example from one of the system")]
[group("run")]
example system example mode="debug":
    cargo run -p {{ system }} --example {{ example }} {{ if mode == "release" { "--release" } else { "" } }}

[doc("Remove build directory")]
[group('clean')]
clean:
    cargo clean

[doc("Rebuild the tool each time a change/file add is detected")]
[group('dev')]
[linux]
debug:
    watchexec -c --stop-signal SIGINT -s SIGINT -e rs,toml -v bacon run

[doc("Rebuild the tool each time a change/file add is detected")]
[group('dev')]
[windows]
debug:
    watchexec -r -w src just run

[doc("Run unit tests for the project")]
[group('dev')]
test: build
    cargo test
