alias r := run
alias rr := run-release

doc:
	cargo doc --open --offline
run:
	cargo run
run-release:
	cargo run --release
