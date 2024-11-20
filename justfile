build:
	RUSTFLAGS="-C strip=none --emit=llvm-ir" cargo rustc --manifest-path=emitter/Cargo.toml --crate-type=cdylib --target=wasm32-unknown-unknown --release --features certora
	RUSTFLAGS="-C strip=none --emit=llvm-ir" cargo rustc --manifest-path=pool-factory/Cargo.toml --crate-type=cdylib --target=wasm32-unknown-unknown --release --features certora
	RUSTFLAGS="-C strip=none --emit=llvm-ir" cargo rustc --manifest-path=backstop/Cargo.toml --crate-type=cdylib --target=wasm32-unknown-unknown --release --features certora
	RUSTFLAGS="-C strip=none --emit=llvm-ir" cargo rustc --manifest-path=pool/Cargo.toml --crate-type=cdylib --target=wasm32-unknown-unknown --release --features certora

build-llvm:
    env RUSTFLAGS="--emit=llvm-ir" cargo build --target=wasm32-unknown-unknown --release
    @echo "See target/wasm32-unknown-unknown/release/deps/pool.wasm.ll"

clean:
    rm -rf target