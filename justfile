set windows-shell := ["powershell.exe"]
export RUST_LOG := "info,wgpu_core=off,wgpu_hal=off"
export RUST_BACKTRACE := "1"

# Displays the list of available commands
@just:
    just --list

# Renders every example in examples/ to out/ as svg and png
render:
    cargo build -r
    cargo run -r -q -- examples/flowchart.json -o out/flowchart.svg
    cargo run -r -q -- examples/sequence.json -o out/sequence.svg
    cargo run -r -q -- examples/class.json -o out/class.svg
    cargo run -r -q -- examples/state.json -o out/state.svg
    cargo run -r -q -- examples/entity_relationship.json -o out/entity_relationship.svg
    cargo run -r -q -- examples/styled.json -o out/styled.svg
    cargo run -r -q -- examples/flowchart.json -o out/flowchart.png

# Renders one document: just draw examples/flowchart.json
draw file="examples/flowchart.json" theme="light":
    cargo run -r -q -- {{ file }} --theme {{ theme }}

# Regenerates the images the README links to
docs-images:
    cargo build -r
    cargo run -r -q -- examples/entity_relationship.json -o docs/images/entity_relationship.svg
    cargo run -r -q -- examples/flowchart.json -o docs/images/flowchart.svg
    cargo run -r -q -- examples/sequence.json -o docs/images/sequence.svg
    cargo run -r -q -- examples/class.json -o docs/images/class.svg
    cargo run -r -q -- examples/state.json -o docs/images/state.svg
    cargo run -r -q -- examples/styled.json -o docs/images/styled.svg

# Builds the project in release mode
build:
    cargo build -r

# Builds the playground bundle into dist/
build-playground:
    trunk build

# Serves the playground at http://127.0.0.1:8080
playground:
    trunk serve --open

# Produces a production playground bundle in dist/
dist:
    trunk build --release

# Runs cargo check and a format check
check:
    cargo check --all-targets
    cargo check -p playground --target wasm32-unknown-unknown
    cargo fmt --all -- --check

# Runs the linter and denies warnings
lint:
    cargo clippy --all-targets -- -D warnings
    cargo clippy -p playground --target wasm32-unknown-unknown -- -D warnings

# Formats the code
format:
    cargo fmt --all

# Runs all tests
test:
    cargo test --all

# Installs the wasm toolchain the playground needs
init-wasm:
    rustup target add wasm32-unknown-unknown
    cargo install --locked trunk
    cargo install --locked wasm-bindgen-cli

# Generates and opens documentation
docs:
    cargo doc --open

# Checks for unused dependencies
udeps:
    cargo machete

# Prints a table of all dependencies and their licenses
licenses:
    cargo license

# Checks for problematic licenses in dependencies
licenses-check:
    cargo deny check licenses

# Generates the third-party license attribution document
licenses-html:
    cargo about generate about.hbs -o THIRD_PARTY_LICENSES.html

# Installs development tools
install-tools:
    cargo install --locked cargo-license
    cargo install --locked cargo-about
    cargo install --locked cargo-deny
    cargo install --locked cargo-machete

# Displays version information for Rust tools
@versions:
    rustc --version
    cargo fmt -- --version
    cargo clippy -- --version
