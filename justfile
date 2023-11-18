test:
  cargo t

build:
  cargo b --release

run: build
  ./target/release/tc-rust
  
