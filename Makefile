clean:
	 find . | grep -E "fp-output.rs|oop-output.rs" | xargs rm -rf

test:
	# Generate the output files with basic tests, test examples 
	cargo test
	# Test the generated files
	cargo test -- --ignored
