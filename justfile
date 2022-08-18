alias r := run
alias rr := run-release
alias f := fmt
alias l := lint

run:
	cargo run 
fmt:
	cargo fmt
lint:
	cargo clippy
doc:
	cargo doc --open --offline
run-release:
	cargo run --release
