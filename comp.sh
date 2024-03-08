move: compile
	mv ./receiver/target/release/receiver ./transport-starter-code-main/4700recv
	mv ./sender/target/release/sender ./transport-starter-code-main/4700send

compile: 
	cd receiver && ~/.cargo/bin/cargo build --release
    cd ..
	cd sender && ~/.cargo/bin/cargo build --release