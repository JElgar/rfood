clean:
	 find . | grep -E "fp-output.rs|oop-output.rs" | xargs rm -rf

test:
	# Generate the output files with basic tests, test examples 
	cargo +nightly test
	# Test the generated files
	cd examples; cargo +nightly test
	cd outputs; cargo +nightly test
