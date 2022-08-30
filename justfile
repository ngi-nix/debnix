alias r := run
alias rr := run-release
alias f := fmt
alias l := lint
alias d := doc
alias uf := update-flake-dependencies
alias uc := update-cargo-dependencies

run:
	cargo run 
fmt:
	cargo fmt
lint:
	cargo clippy
	nix run nixpkgs#typos
doc:
	cargo doc --open --offline
run-release:
	cargo run --release

update-flake-dependencies:
	nix flake update --commit-lock-file

# Update and then commit the `Cargo.lock` file
update-cargo-dependencies:
	cargo update
	git add Cargo.lock
	git commit Cargo.lock -m "update(cargo): `Cargo.lock`"

error:
	export RUST_LOG=error
info:
	export RUST_LOG=info
debug:
	export RUST_LOG=debug

prettify-json:
	fd ".-debnix.json" outputs --exec jq . {}
