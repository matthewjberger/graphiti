# The document format

A document is one object with one field:

```json
{ "kind": { "type": "flowchart", "...": "..." } }
```

In Rust that is:

```rust
pub struct Diagram {
    pub kind: DiagramKind,
}

#[serde(tag = "type", rename_all = "snake_case")]
pub enum DiagramKind {
    Flowchart(Flowchart),
    Sequence(Sequence),
    Class(ClassDiagram),
    State(StateDiagram),
    EntityRelationship(EntityRelationship),
}
```

The enum is internally tagged, so the variant's struct fields sit next to
`type` rather than nesting under another key. Every field has a `serde` default,
so a document only carries what it wants to say.

## The style block

Every kind takes an optional `style` object. Omit it and the document renders
with the theme the caller picked. Set any part of it and the generator honors
that instead, so a document can carry its own look without the reader passing
flags.

```json
{
  "kind": {
    "type": "class",
    "style": {
      "theme": "dark",
      "compact": true,
      "monospace": true,
      "zoom": 1.1,
      "corner_radius": 3,
      "palette": {
        "background": "#0F1116",
        "primary": { "fill": "#1B2E4A", "border": "#4C86D8", "strong": "#7FB2FF" }
      }
    },
    "classes": []
  }
}
```

| Field | Meaning |
| --- | --- |
| `theme` | Base palette: `light`, `dark`, or `mono`. Overrides the caller's choice. |
| `zoom` | Scales every metric, clamped to 0.25 through 4. Text, padding, and gaps grow together. |
| `compact` | Tightens padding, gaps, and margins. Defaults to `false`. |
| `monospace` | Draws class and entity rows in the monospace face. Defaults to `false`. |
| `label_size` | Base label size in pixels. Detail and member sizes scale with it. |
| `title_size`, `detail_size` | Sizes for the title and for secondary text. |
| `node_padding` | Horizontal padding inside a node; vertical padding follows at 72 percent. |
| `node_min_width`, `node_min_height` | Floors for node size. |
| `rank_gap`, `sibling_gap` | Space between ranks and between neighbors in a rank. |
| `corner_radius`, `border_width`, `edge_width`, `arrow_size` | Line and corner weights. |
| `margin` | Space between the drawing and the canvas edge. |
| `line_height` | Multiple of the font size used for line spacing. |
| `palette` | Per-role color overrides, described below. |

Colors are `#RRGGBB` or `#RRGGBBAA` strings, converted from sRGB to linear on
the way in. `palette` takes `background`, `surface`, `surface_alt`, `border`,
`text`, `text_muted`, `edge`, `group_fill`, and `group_border`, plus a nested
object per accent role (`primary`, `success`, `warning`, `danger`, `info`,
`muted`) with `fill`, `border`, `strong`, and `text`. Anything left out keeps
the base theme's value.

Unknown keys inside `style` are rejected rather than ignored, so a typo in a
document is a parse error instead of a silently missing override.

## Shared vocabulary

These appear across kinds and all serialize as `snake_case`.

| Type | Values |
| --- | --- |
| `Direction` | `down`, `up`, `right`, `left` |
| `NodeShape` | `rectangle`, `rounded`, `stadium`, `circle`, `diamond`, `hexagon`, `parallelogram`, `cylinder`, `subroutine`, `note` |
| `LineStyle` | `solid`, `dashed`, `dotted`, `thick` |
| `ArrowHead` | `none`, `arrow`, `open`, `hollow_triangle`, `diamond`, `hollow_diamond`, `circle`, `hollow_circle`, `bar`, `crows_foot`, `crows_foot_one`, `crows_foot_zero_or_one`, `crows_foot_zero_or_many`, `crows_foot_one_or_many` |
| `EdgeRouting` | `orthogonal`, `curved`, `straight` |
| `Accent` | `neutral`, `primary`, `success`, `warning`, `danger`, `info`, `muted` |
| `Visibility` | `public`, `private`, `protected`, `package` |

`Accent` is a semantic role, not a color. The theme decides what each role
looks like, which is why documents stay readable in both palettes and why no
document carries hex codes.

## Flowchart

```json
{
  "kind": {
    "type": "flowchart",
    "title": "Release pipeline",
    "direction": "down",
    "routing": "orthogonal",
    "nodes": [
      { "id": "push", "label": "Push to main", "shape": "stadium", "accent": "primary" },
      { "id": "build", "label": "Build", "detail": "cargo build --release" }
    ],
    "edges": [
      { "from": "push", "to": "build", "label": "on push", "style": "solid", "head": "arrow" }
    ],
    "groups": [
      { "id": "ci", "label": "Continuous integration", "nodes": ["build"] }
    ]
  }
}
```

`detail` is a second line of smaller muted text inside the node. `groups` draw a
labeled container around their members, and the layout keeps other nodes out of
that container's span.

## Sequence

`participants` are the lifelines, in the order they should appear. `steps` is a
list of tagged steps, and fragments nest steps inside themselves.

```json
{
  "kind": {
    "type": "sequence",
    "participants": [
      { "id": "user", "label": "Shopper", "kind": "actor" },
      { "id": "api", "label": "Orders API" }
    ],
    "steps": [
      { "step": "message", "from": "user", "to": "api", "label": "POST /orders", "activate": true },
      { "step": "note", "text": "Idempotency key required.", "over": ["api"] },
      {
        "step": "fragment",
        "kind": "alt",
        "label": "authorized",
        "steps": [{ "step": "message", "from": "api", "to": "user", "label": "201", "kind": "reply" }],
        "branches": [
          { "label": "declined", "steps": [{ "step": "message", "from": "api", "to": "user", "label": "402", "kind": "reply" }] }
        ]
      }
    ]
  }
}
```

`step` is `message`, `note`, `fragment`, or `divider`. A message `kind` is
`sync`, `async`, `reply`, `create`, or `destroy`. `activate` opens an activation
bar on the receiver, and `deactivate` closes the bar on the sender, which is the
participant finishing its work as it replies. Bars still open at the end of the
diagram run to the bottom of the lifeline. Fragment `kind` is `loop`,
`alt`, `opt`, `par`, `critical`, or `break`. `ParticipantKind` is `participant`,
`actor`, `database`, or `boundary`.

## Class

```json
{
  "kind": {
    "type": "class",
    "classes": [
      {
        "id": "surface",
        "name": "Surface",
        "stereotype": "interface",
        "fields": [{ "name": "id", "type_name": "u32", "visibility": "private" }],
        "methods": [{ "name": "present", "type_name": "Result", "is_abstract": true }]
      }
    ],
    "relations": [
      { "from": "vulkan", "to": "surface", "kind": "realization", "to_cardinality": "1" }
    ]
  }
}
```

`RelationKind` is `association`, `inheritance`, `realization`, `composition`,
`aggregation`, or `dependency`, and each picks its own UML line and end
decoration. Inheritance and realization are laid out with the parent above the
child regardless of which way the relation is written.

## State

```json
{
  "kind": {
    "type": "state",
    "states": [
      { "id": "begin", "kind": "start" },
      { "id": "running", "label": "Downloading", "description": ["do / stream chunks"] },
      { "id": "pick", "kind": "choice", "label": "capacity?" },
      { "id": "finish", "kind": "end" }
    ],
    "transitions": [
      { "from": "begin", "to": "running", "label": "start" }
    ]
  }
}
```

`StateKind` is `simple`, `start`, `end`, `choice`, `fork`, or `join`. The marker
kinds draw as filled dots, diamonds, and bars instead of boxes. `description`
lines render below a divider inside the state.

## Entity relationship

```json
{
  "kind": {
    "type": "entity_relationship",
    "direction": "right",
    "entities": [
      {
        "id": "order",
        "name": "order",
        "attributes": [
          { "name": "id", "type_name": "uuid", "key": "primary" },
          { "name": "customer_id", "type_name": "uuid", "key": "foreign" }
        ]
      }
    ],
    "relationships": [
      {
        "from": "customer",
        "to": "order",
        "label": "places",
        "from_cardinality": "exactly_one",
        "to_cardinality": "zero_or_many",
        "identifying": true
      }
    ]
  }
}
```

`KeyKind` is `none`, `primary`, `foreign`, or `unique`, and shows as a `PK`,
`FK`, or `UK` badge. `Cardinality` is `exactly_one`, `zero_or_one`,
`zero_or_many`, or `one_or_many`, drawn as crow's foot notation on both ends.
A relationship that is not `identifying` uses a dashed line.
