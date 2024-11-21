build:
	RUSTFLAGS="-C strip=none --emit=llvm-ir" cargo build --manifest-path=emitter/Cargo.toml --target=wasm32-unknown-unknown --release --features certora
	RUSTFLAGS="-C strip=none --emit=llvm-ir" cargo build --manifest-path=pool-factory/Cargo.toml --target=wasm32-unknown-unknown --release --features certora
	RUSTFLAGS="-C strip=none --emit=llvm-ir" cargo build --manifest-path=backstop/Cargo.toml --target=wasm32-unknown-unknown --release --features certora
	RUSTFLAGS="-C strip=none --emit=llvm-ir" cargo build --manifest-path=pool/Cargo.toml --target=wasm32-unknown-unknown --release --features certora

build-llvm:
    env RUSTFLAGS="--emit=llvm-ir" cargo build --target=wasm32-unknown-unknown --release
    @echo "See target/wasm32-unknown-unknown/release/deps/pool.wasm.ll"

clean:
    rm -rf target