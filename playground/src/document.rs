use graphiti::schema::{self, Diagram, DiagramKind};
use leptos::prelude::*;

#[derive(Clone, Copy)]
pub struct Document {
    pub source: RwSignal<String>,
    pub parsed: Memo<Result<Diagram, String>>,
}

pub fn new_document(source: RwSignal<String>) -> Document {
    Document {
        source,
        parsed: Memo::new(move |_| schema::parse(&source.get()).map_err(|error| error.to_string())),
    }
}

pub fn edit(document: Document, mutate: impl FnOnce(&mut Diagram)) {
    let Ok(mut current) = document.parsed.get_untracked() else {
        return;
    };
    mutate(&mut current);
    write(document, &current);
}

pub fn set_kind(document: Document, name: &str) {
    let Some(mut kind) = schema::kind_from_name(name) else {
        return;
    };
    if let Ok(current) = document.parsed.get_untracked() {
        *schema::title_mut(&mut kind) = schema::title(&current.kind).map(str::to_string);
        *schema::style_mut(&mut kind) = schema::style(&current.kind).clone();
    }
    write(document, &Diagram { kind });
}

pub fn kind_name(document: Document) -> &'static str {
    document
        .parsed
        .with(|value| {
            value
                .as_ref()
                .ok()
                .map(|current| schema::kind_name(&current.kind))
        })
        .unwrap_or("flowchart")
}

pub fn read<T: Default>(document: Document, take: impl FnOnce(&DiagramKind) -> T) -> T {
    document.parsed.with(|value| match value {
        Ok(current) => take(&current.kind),
        Err(_) => T::default(),
    })
}

fn write(document: Document, diagram: &Diagram) {
    if let Ok(text) = schema::to_json(diagram) {
        document.source.set(text);
    }
}
