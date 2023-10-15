# graphiti

[<img alt="github" src="https://img.shields.io/badge/github-matthewjberger/graphiti-8da0cb?style=for-the-badge&labelColor=555555&logo=github" height="20">](https://github.com/matthewjberger/graphiti)
[<img alt="crates.io" src="https://img.shields.io/crates/v/graphiti.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/graphiti)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-graphiti-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" height="20">](https://docs.rs/graphiti)

`graphiti` provides a rust macro dsl and builder for creating and describing
arbitrary sets of related data, represented by a `Description` type.

A common use case is for modeling physical simulations, where you would want to leverage an entity component system but want the benefits of describing relationships between simulation members using directed graphs.

With `graphiti`, simulations can be described using a type-safe rust macro dsl that drives a builder pattern constructing a final `Description` type. With this `Description`, the data is organized and easily accessible for business logic that may want to use the stored data. This could be for a game (ECS + Scenegraphs), hardware simulations, etc.

## Usage

Add this to your `Cargo.toml`:

```toml
graphiti = "0.1.1"
```

Example:

```rust
fn main() {
    let description = graphiti::describe! {
        nodes: {
            device: "device",
            safety: "safety",
            controller: "controller",
            power: "power",
            control: "control",
            io: "io"
        },
        edges: {
            "config_standard": {
                device: [safety, controller, power, control, io],
                safety: [controller, power]
            },
            "config_alternate": {
                device: [controller, control, io],
                controller: [power]
            }
        }
    };
    println!("{description:#?}");
}
```
