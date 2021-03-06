clean:
	 find . | grep -E "fp-output.rs|oop-output.rs" | xargs rm -rf

build:
	 cargo +nightly build

build-pkg:
	wasm-pack build --target nodejs

test:
	# Generate the output files with basic tests, test examples 
	cargo +nightly test
	# Test the generated files
	cd examples; cargo +nightly test
	cd outputs; cargo +nightly test
