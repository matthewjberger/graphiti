use graphiti::layout::approximate_measure;
use graphiti::{build_scene, schema, theme};

fn example_sources() -> Vec<(String, String)> {
    let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("examples");
    let mut sources = Vec::new();
    for entry in std::fs::read_dir(directory).expect("examples directory missing") {
        let path = entry.expect("unreadable entry").path();
        if path.extension().and_then(|value| value.to_str()) == Some("json") {
            let name = path
                .file_stem()
                .and_then(|value| value.to_str())
                .unwrap_or_default()
                .to_string();
            let source = std::fs::read_to_string(&path).expect("unreadable example");
            sources.push((name, source));
        }
    }
    assert!(!sources.is_empty(), "no examples found");
    sources
}

#[test]
fn every_example_round_trips_through_json() {
    for (name, source) in example_sources() {
        let diagram = schema::parse(&source).unwrap_or_else(|error| panic!("{name}: {error}"));
        let encoded = schema::to_json(&diagram).expect("serialization failed");
        let decoded = schema::parse(&encoded).unwrap_or_else(|error| panic!("{name}: {error}"));
        assert_eq!(
            schema::kind_name(&diagram.kind),
            schema::kind_name(&decoded.kind),
            "{name} changed kind across a round trip"
        );
    }
}

#[test]
fn every_example_lays_out_inside_its_canvas() {
    for (name, source) in example_sources() {
        let diagram = schema::parse(&source).expect("parse failed");
        let scene = build_scene(&diagram, &theme::theme_light(), &mut approximate_measure);

        assert!(
            scene.size.x.is_finite() && scene.size.y.is_finite(),
            "{name} produced a non-finite canvas"
        );
        assert!(
            scene.size.x > 64.0 && scene.size.y > 64.0,
            "{name} produced a degenerate canvas of {}x{}",
            scene.size.x,
            scene.size.y
        );

        let slack = 8.0;
        for rect in &scene.rects {
            assert!(
                rect.position.x > -slack && rect.position.y > -slack,
                "{name} placed a rect outside the canvas at {:?}",
                rect.position
            );
            assert!(
                rect.position.x + rect.size.x < scene.size.x + slack,
                "{name} placed a rect past the right edge"
            );
            assert!(
                rect.position.y + rect.size.y < scene.size.y + slack,
                "{name} placed a rect past the bottom edge"
            );
        }
        for label in &scene.labels {
            assert!(
                label.position.x.is_finite() && label.position.y.is_finite(),
                "{name} placed a label at a non-finite position"
            );
            assert!(
                !label.text.is_empty(),
                "{name} produced an empty label entry"
            );
        }
        for stroke in &scene.strokes {
            assert!(
                stroke.points.len() >= 2,
                "{name} produced a degenerate edge"
            );
            for point in &stroke.points {
                assert!(
                    point.x.is_finite() && point.y.is_finite(),
                    "{name} produced a non-finite edge point"
                );
            }
        }
    }
}

#[test]
fn themes_agree_on_geometry() {
    for (name, source) in example_sources() {
        let diagram = schema::parse(&source).expect("parse failed");
        let light = build_scene(&diagram, &theme::theme_light(), &mut approximate_measure);
        let dark = build_scene(&diagram, &theme::theme_dark(), &mut approximate_measure);
        assert_eq!(
            light.rects.len(),
            dark.rects.len(),
            "{name} drew a different number of rects per theme"
        );
        assert_eq!(
            light.size, dark.size,
            "{name} sized its canvas differently per theme"
        );
    }
}

#[test]
fn unknown_node_references_are_dropped_instead_of_panicking() {
    let source = r#"{
      "kind": {
        "type": "flowchart",
        "nodes": [{ "id": "a", "label": "A" }],
        "edges": [{ "from": "a", "to": "missing" }, { "from": "ghost", "to": "a" }]
      }
    }"#;
    let diagram = schema::parse(source).expect("parse failed");
    let scene = build_scene(&diagram, &theme::theme_light(), &mut approximate_measure);
    assert_eq!(scene.rects.len(), 1);
    assert!(scene.strokes.is_empty());
}
