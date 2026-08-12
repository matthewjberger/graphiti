pub struct Sample {
    pub name: &'static str,
    pub source: &'static str,
}

pub const SAMPLES: &[Sample] = &[
    Sample {
        name: "Flowchart",
        source: include_str!("../../examples/flowchart.json"),
    },
    Sample {
        name: "Sequence",
        source: include_str!("../../examples/sequence.json"),
    },
    Sample {
        name: "Class",
        source: include_str!("../../examples/class.json"),
    },
    Sample {
        name: "State",
        source: include_str!("../../examples/state.json"),
    },
    Sample {
        name: "Entity relationship",
        source: include_str!("../../examples/entity_relationship.json"),
    },
    Sample {
        name: "Styled",
        source: include_str!("../../examples/styled.json"),
    },
];

pub fn source_for(name: &str) -> &'static str {
    SAMPLES
        .iter()
        .find(|sample| sample.name == name)
        .map(|sample| sample.source)
        .unwrap_or(SAMPLES[0].source)
}
