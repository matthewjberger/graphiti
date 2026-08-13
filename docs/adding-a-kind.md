# Adding a diagram kind

Say the new kind is a mindmap. Seven steps, and the compiler names five of them.

## 1. Describe it as data

`src/schema/mindmap.rs`:

```rust
use crate::schema::common::{Accent, Direction, Style};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Mindmap {
    #[serde(
        default,
        skip_serializing_if = "crate::schema::common::style_is_default"
    )]
    pub style: Style,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(default)]
    pub direction: Direction,
    #[serde(default)]
    pub nodes: Vec<MindmapNode>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct MindmapNode {
    pub id: String,
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub children: Vec<String>,
    #[serde(default)]
    pub accent: Accent,
}
```

Give every field a `serde` default so short documents stay legal, and reach for
the shared vocabulary in `schema::common` before inventing new enums. Carry
`style` so the kind honors the style block like the others, and derive
`PartialEq` so the playground can tell one document from another.

## 2. Add the variant

`src/schema.rs`:

```rust
pub mod mindmap;
pub use mindmap::Mindmap;

pub enum DiagramKind {
    // ...
    Mindmap(Mindmap),
}
```

Then follow the compiler through `kind_name`, `kind_from_name`, `kind_names`,
`style`, `style_mut`, `title`, and `title_mut`, which are exhaustive matches on
purpose.

## 3. Write the generator

`src/layout/mindmap.rs` exposes one free function:

```rust
pub fn generate(data: &Mindmap, theme: &Theme, measure: Measure) -> Scene
```

Measure the text, decide the sizes, place things, and push primitives into the
`Scene`. If the kind is node-and-edge shaped, hand node sizes and edge pairs to
`layout::graph::layout_layered` and let it do the placement, then draw edges with
`layout::edges::draw_layered_edge`; that is what the other four kinds do and it
is where the layout quality lives. If it has its own geometry, like sequence
diagrams do, walk your own cursor instead.

Rules that keep a generator consistent with the rest:

- Take colors from the theme through `accent_colors`, never literal colors.
- Size every box from measured text, never from a guess.
- Use the `LAYER_*` constants for depth so containers, edges, nodes, and labels
  stack the same way everywhere.
- Set `scene.size` to the full canvas including margins, and keep every
  coordinate inside it.

## 4. Dispatch to it

`src/layout.rs`:

```rust
pub mod mindmap;

DiagramKind::Mindmap(data) => mindmap::generate(data, theme, measure),
```

That match is the whole extension point. Nothing in `render`, `scene`,
`geometry`, or the CLI changes, because they only ever see a `Scene`.

## 5. Check its references

`src/validate.rs` reports the mistakes a generator would otherwise swallow:
duplicate ids, blank ids, and edges that point at something absent. Add an arm
for the new kind so a typo shows up as a message instead of a missing edge. The
CLI prints these to stderr and the playground lists them under the editor.

## 6. Give it a form

`playground/src/builder/mindmap.rs` exposes one free function:

```rust
pub fn view(document: Document, search: RwSignal<String>) -> impl IntoView
```

Copy the shape of `builder/state_diagram.rs`, the smallest one: a `with` reader
and a `change` writer for the kind, one pair per collection, and the widgets from
`playground/src/form.rs`. Add enum tables to `playground/src/options.rs`, then
add the arm to the match in `playground/src/builder.rs`.

## 7. Prove it

Drop a document in `examples/mindmap.json`. The tests in `tests/examples.rs`
sweep that directory, so the new kind is immediately checked for round tripping
through JSON, for laying out inside its canvas with finite coordinates, and for
producing the same geometry under both themes. Add it to `playground/src/samples.rs`
and the table in the README, then render it:

```sh
cargo run -r -- examples/mindmap.json -o out/mindmap.png
```
