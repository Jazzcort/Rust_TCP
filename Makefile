move: compile
	mv ./receiver/target/release/receiver ./4700recv
	mv ./sender/target/release/sender ./4700send

compile: client
	cd receiver && ~/.cargo/bin/cargo build --release
	cd ../sender && ~/.cargo/bin/cargo build --release

# Thanks for Luke Jianu
client: 
	curl https://sh.rustup.rs -sSf | sh -s -- -y \
	&& ~/.cargo/bin/rustup install --profile=minimal 1.75.0 \
	&& ~/.cargo/bin/rustup default 1.75.0 