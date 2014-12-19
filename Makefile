all:
	cargo build
test:
	cargo test
clean:
	-rm -rf target/
.PHONY: all test clean
