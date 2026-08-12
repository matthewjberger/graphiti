# graphiti

Diagrams as data. A document goes in as JSON, a rendered image comes out.

Every diagram is one struct with a single `kind` field. That field is a tagged
enum, and each variant carries a plain struct describing that kind of diagram.
So the format is the Rust data model, and adding a diagram kind means adding a
variant and a generator for it.

Layout and rendering are ours end to end: a layered graph layout with crossing
minimization, orthogonal edge routing with rounded corners, and geometry drawn
through [nightshade](https://github.com/matthewjberger/nightshade) with an
orthographic camera, supersampled and downsampled for clean edges.

## A document and its image

```json
{
  "kind": {
    "type": "entity_relationship",
    "title": "Store schema",
    "direction": "right",
    "entities": [
      {
        "id": "customer",
        "name": "customer",
        "accent": "primary",
        "attributes": [
          { "name": "id", "type_name": "uuid", "key": "primary" },
          { "name": "email", "type_name": "text", "key": "unique" }
        ]
      },
      {
        "id": "order",
        "name": "order",
        "accent": "info",
        "attributes": [
          { "name": "id", "type_name": "uuid", "key": "primary" },
          { "name": "customer_id", "type_name": "uuid", "key": "foreign" },
          { "name": "total_cents", "type_name": "bigint" }
        ]
      }
    ],
    "relationships": [
      {
        "from": "customer",
        "to": "order",
        "label": "places",
        "from_cardinality": "exactly_one",
        "to_cardinality": "zero_or_many"
      }
    ]
  }
}
```

```sh
graphiti schema.json -o schema.png
```

![Store schema](docs/images/entity_relationship.png)

## Running it

```sh
# render a document; the image lands next to it unless you pass -o
cargo run -r -- examples/flowchart.json -o out/flowchart.png

# the dark palette
cargo run -r -- examples/flowchart.json --theme dark -o out/flowchart.png

# render every example into out/
just render
```

Options: `-o/--output` for the destination, `--theme light|dark`, and
`--supersample 1..4` for how much the renderer oversamples before downsampling
(2 by default).

The interactive playground runs the same schema, layout, and renderer in the
browser, with the engine on an `OffscreenCanvas` in a web worker:

```sh
just init-wasm    # once: wasm target, trunk, wasm-bindgen, wasm-opt
just playground   # serves http://127.0.0.1:8080
```

> The playground needs WebGPU, which every Chromium browser and Firefox 141+
> support.

## Diagram kinds

| Kind                  | `type`                 | Example                                    |
| --------------------- | ---------------------- | ------------------------------------------ |
| Flowchart             | `flowchart`            | [flowchart](examples/flowchart.json)       |
| Sequence              | `sequence`             | [sequence](examples/sequence.json)         |
| Class                 | `class`                | [class](examples/class.json)               |
| State                 | `state`                | [state](examples/state.json)               |
| Entity relationship   | `entity_relationship`  | [er](examples/entity_relationship.json)    |

![Checkout with retry](docs/images/sequence.png)

More on the data model, the layout pipeline, and how to add a kind:
[docs/format.md](docs/format.md), [docs/layout.md](docs/layout.md),
[docs/adding-a-kind.md](docs/adding-a-kind.md).

## Prerequisites

* A GPU with Vulkan, Metal, or DX12, since rendering goes through wgpu
* [just](https://github.com/casey/just) for the recipes above
* [trunk](https://trunkrs.dev/) and the wasm target for the playground

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
