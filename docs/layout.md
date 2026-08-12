# From document to image

Four stages, each a plain function over plain data:

```
JSON -> Diagram -> Scene -> ECS entities -> PNG
        schema     layout    render         nightshade
```

## Diagram to Scene

`layout::build_scene(&diagram, &theme, &mut measure)` matches on the kind and
calls that kind's generator. The generator returns a `Scene`: a
resolution-independent draw list held as parallel vectors of rects, polygons,
circles, strokes, and labels, each carrying a `Paint` and a layer depth. Nothing
in a `Scene` names a GPU or an engine type, which is what lets the same scene
feed a file, a canvas, or a test.

`measure` is a `&mut dyn FnMut(&str, f32) -> f32` the caller supplies. The CLI
and the playground pass real text measurement from the engine's font engine, so
a box is sized to the text that will actually be drawn in it. Tests pass
`layout::approximate_measure`, which needs no fonts.

## The layered layout

Flowchart, class, state, and entity relationship diagrams all go through
`layout::graph::layout_layered`, which takes node sizes and edges and returns
positions plus per-edge waypoints:

1. **Break cycles.** A depth-first pass reverses back edges so the rest of the
   pipeline sees a DAG. Reversed edges are drawn in their original direction.
2. **Rank.** A longest-path layering puts every node one rank past its deepest
   predecessor.
3. **Add lanes.** An edge spanning more than one rank gets a virtual slot in each
   rank between its ends, so long edges have somewhere to run.
4. **Order within ranks.** Barycenter sweeps down and up, then adjacent-pair
   transposition, keeping whichever ordering crossed the fewest edges.
5. **Keep groups together.** Each rank is reordered so members of the same group
   sit next to each other.
6. **Place across the rank.** Positions relax toward the weighted median of each
   node's neighbors, with virtual slots weighted heavily so long edges come out
   straight, then overlaps are resolved in order so nothing collides.
7. **Reserve group space.** Nodes that are not members of a group are pushed out
   of the span the group's container will occupy.
8. **Place along the ranks.** Each rank gets the height of its tallest node plus
   the rank gap, flipped for `up` and `left`.

Sequence diagrams do not use this: lifelines are fixed columns, and the vertical
cursor advances per step, so their generator walks the steps directly.

## Edges

`layout::edges::draw_layered_edge` turns waypoints into a drawn path. It picks
the side to leave and enter from by comparing the endpoints along the rank axis,
so a back edge exits the near side instead of dragging a line across its own
node. Orthogonal routing inserts a jog at the midpoint between ranks and rounds
every corner. Parallel edges between the same pair of nodes get a lane offset so
a bidirectional pair reads as two lines, and their labels shift by the same
offset. Edge labels land on the midpoint of the path's longest straight run,
which keeps them off nodes and corners.

## Scene to image

`render::populate_world` walks the scene and spawns engine entities:

- Filled shapes are triangulated on the CPU, ear clipping for polygons and fans
  for discs and rings, then merged into one mesh per color and layer and spawned
  as a custom mesh with an unlit material.
- Strokes become quads per segment with round joins and caps, and dashes are cut
  before triangulation.
- Labels become world-space billboard text.
- The camera is orthographic and fitted to the scene, so one scene unit is one
  pixel at supersample 1.

Scene space has y pointing down with the origin at the top left. The renderer
flips y on the way to world space and reverses triangle winding to match, which
is why every triangle the geometry module emits is wound the same way.

The engine's post-processing is turned off for the capture: unlit shading, no
bloom, no ambient occlusion, no temporal antialiasing, and no tonemapping, so a
color in the theme is the color in the file. Antialiasing comes from rendering
at `--supersample` times the target size and downsampling with a Lanczos filter.
