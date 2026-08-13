use graphiti::schema::DiagramKind;
use graphiti::schema::entity_relationship::{Attribute, Entity, EntityRelationship, Relationship};
use leptos::prelude::*;

use crate::document::{Document, edit, read};
use crate::form::{Card, Choice, Reference, Section, TextInput, Toggle, matches, optional};
use crate::options::{ACCENTS, CARDINALITIES, DIRECTIONS, KEY_KINDS};

pub fn view(document: Document, search: RwSignal<String>) -> impl IntoView {
    let identifiers = Signal::derive(move || ids(document));
    view! {
        <div class="row">
            <Choice
                label="Direction"
                options=DIRECTIONS
                value=Signal::derive(move || with(document, |data| data.direction))
                change=Callback::new(move |value| {
                    change(document, move |data| data.direction = value)
                })
            />
        </div>

        <Section
            title=Signal::derive(move || {
                format!("Entities ({})", with(document, |data| data.entities.len()))
            })
            add=Callback::new(move |_| {
                change(
                    document,
                    |data| {
                        let id = format!("entity{}", data.entities.len() + 1);
                        data.entities.push(Entity { name: id.clone(), id, ..Entity::default() });
                    },
                )
            })
        >
            <For
                each=move || visible_entities(document, search)
                key=|index| *index
                children=move |index| entity_card(document, index)
            />
        </Section>

        <Section
            title=Signal::derive(move || {
                format!("Relationships ({})", with(document, |data| data.relationships.len()))
            })
            add=Callback::new(move |_| {
                change(
                    document,
                    |data| {
                        let first = data
                            .entities
                            .first()
                            .map(|entry| entry.id.clone())
                            .unwrap_or_default();
                        data.relationships
                            .push(Relationship {
                                from: first.clone(),
                                to: first,
                                identifying: true,
                                ..Relationship::default()
                            });
                    },
                )
            })
        >
            <For
                each=move || visible_relationships(document, search)
                key=|index| *index
                children=move |index| relationship_card(document, index, identifiers)
            />
        </Section>
    }
}

fn entity_card(document: Document, index: usize) -> impl IntoView {
    view! {
        <Card
            title=format!("Entity {}", index + 1)
            remove=Callback::new(move |_| {
                change(
                    document,
                    move |data| {
                        if index < data.entities.len() {
                            data.entities.remove(index);
                        }
                    },
                )
            })
        >
            <div class="grid">
                <TextInput
                    label="Id"
                    value=Signal::derive(move || entity(document, index, |entry| entry.id.clone()))
                    change=Callback::new(move |text: String| rename(document, index, text))
                />
                <TextInput
                    label="Name"
                    value=Signal::derive(move || entity(document, index, |entry| entry.name.clone()))
                    change=Callback::new(move |text: String| {
                        change_entity(document, index, move |entry| entry.name = text)
                    })
                />
                <Choice
                    label="Accent"
                    options=ACCENTS
                    value=Signal::derive(move || entity(document, index, |entry| entry.accent))
                    change=Callback::new(move |value| {
                        change_entity(document, index, move |entry| entry.accent = value)
                    })
                />
            </div>
            <Section
                title=Signal::derive(move || {
                    format!(
                        "Attributes ({})",
                        entity(document, index, |entry| entry.attributes.len()),
                    )
                })
                add=Callback::new(move |_| {
                    change_entity(
                        document,
                        index,
                        move |entry| {
                            let count = entry.attributes.len();
                            entry
                                .attributes
                                .push(Attribute {
                                    name: format!("field{}", count + 1),
                                    ..Attribute::default()
                                });
                        },
                    )
                })
            >
                <For
                    each=move || 0..entity(document, index, |entry| entry.attributes.len())
                    key=|position| *position
                    children=move |position| attribute_card(document, index, position)
                />
            </Section>
        </Card>
    }
}

fn attribute_card(document: Document, index: usize, position: usize) -> impl IntoView {
    view! {
        <Card
            title=(position + 1).to_string()
            remove=Callback::new(move |_| {
                change_entity(
                    document,
                    index,
                    move |entry| {
                        if position < entry.attributes.len() {
                            entry.attributes.remove(position);
                        }
                    },
                )
            })
        >
            <div class="grid">
                <TextInput
                    label="Name"
                    value=Signal::derive(move || {
                        attribute(document, index, position, |value| value.name.clone())
                    })
                    change=Callback::new(move |text: String| {
                        change_attribute(document, index, position, move |value| value.name = text)
                    })
                />
                <TextInput
                    label="Type"
                    placeholder="text"
                    value=Signal::derive(move || {
                        attribute(
                            document,
                            index,
                            position,
                            |value| value.type_name.clone().unwrap_or_default(),
                        )
                    })
                    change=Callback::new(move |text: String| {
                        change_attribute(
                            document,
                            index,
                            position,
                            move |value| value.type_name = optional(text),
                        )
                    })
                />
                <Choice
                    label="Key"
                    options=KEY_KINDS
                    value=Signal::derive(move || {
                        attribute(document, index, position, |value| value.key)
                    })
                    change=Callback::new(move |chosen| {
                        change_attribute(
                            document,
                            index,
                            position,
                            move |value| value.key = chosen,
                        )
                    })
                />
                <TextInput
                    label="Comment"
                    value=Signal::derive(move || {
                        attribute(
                            document,
                            index,
                            position,
                            |value| value.comment.clone().unwrap_or_default(),
                        )
                    })
                    change=Callback::new(move |text: String| {
                        change_attribute(
                            document,
                            index,
                            position,
                            move |value| value.comment = optional(text),
                        )
                    })
                />
            </div>
        </Card>
    }
}

fn relationship_card(
    document: Document,
    index: usize,
    identifiers: Signal<Vec<String>>,
) -> impl IntoView {
    view! {
        <Card
            title=format!("Relationship {}", index + 1)
            remove=Callback::new(move |_| {
                change(
                    document,
                    move |data| {
                        if index < data.relationships.len() {
                            data.relationships.remove(index);
                        }
                    },
                )
            })
        >
            <div class="grid">
                <Reference
                    label="From"
                    options=identifiers
                    value=Signal::derive(move || {
                        relationship(document, index, |entry| entry.from.clone())
                    })
                    change=Callback::new(move |text: String| {
                        change_relationship(document, index, move |entry| entry.from = text)
                    })
                />
                <Reference
                    label="To"
                    options=identifiers
                    value=Signal::derive(move || {
                        relationship(document, index, |entry| entry.to.clone())
                    })
                    change=Callback::new(move |text: String| {
                        change_relationship(document, index, move |entry| entry.to = text)
                    })
                />
                <TextInput
                    label="Label"
                    value=Signal::derive(move || {
                        relationship(document, index, |entry| {
                            entry.label.clone().unwrap_or_default()
                        })
                    })
                    change=Callback::new(move |text: String| {
                        change_relationship(
                            document,
                            index,
                            move |entry| entry.label = optional(text),
                        )
                    })
                />
                <Choice
                    label="From cardinality"
                    options=CARDINALITIES
                    value=Signal::derive(move || {
                        relationship(document, index, |entry| entry.from_cardinality)
                    })
                    change=Callback::new(move |value| {
                        change_relationship(
                            document,
                            index,
                            move |entry| entry.from_cardinality = value,
                        )
                    })
                />
                <Choice
                    label="To cardinality"
                    options=CARDINALITIES
                    value=Signal::derive(move || {
                        relationship(document, index, |entry| entry.to_cardinality)
                    })
                    change=Callback::new(move |value| {
                        change_relationship(
                            document,
                            index,
                            move |entry| entry.to_cardinality = value,
                        )
                    })
                />
            </div>
            <Toggle
                label="Identifying"
                value=Signal::derive(move || {
                    relationship(document, index, |entry| entry.identifying)
                })
                change=Callback::new(move |flag: bool| {
                    change_relationship(document, index, move |entry| entry.identifying = flag)
                })
            />
        </Card>
    }
}

fn ids(document: Document) -> Vec<String> {
    with(document, |data| {
        data.entities.iter().map(|entry| entry.id.clone()).collect()
    })
}

fn visible_entities(document: Document, search: RwSignal<String>) -> Vec<usize> {
    let needle = search.get();
    with(document, |data| {
        data.entities
            .iter()
            .enumerate()
            .filter(|(_, entry)| matches(&needle, &[&entry.id, &entry.name]))
            .map(|(index, _)| index)
            .collect()
    })
}

fn visible_relationships(document: Document, search: RwSignal<String>) -> Vec<usize> {
    let needle = search.get();
    with(document, |data| {
        data.relationships
            .iter()
            .enumerate()
            .filter(|(_, entry)| {
                matches(
                    &needle,
                    &[&entry.from, &entry.to, entry.label.as_deref().unwrap_or("")],
                )
            })
            .map(|(index, _)| index)
            .collect()
    })
}

fn with<T: Default>(document: Document, take: impl FnOnce(&EntityRelationship) -> T) -> T {
    read(document, |kind| match kind {
        DiagramKind::EntityRelationship(data) => take(data),
        _ => T::default(),
    })
}

fn change(document: Document, mutate: impl FnOnce(&mut EntityRelationship)) {
    edit(document, |diagram| {
        if let DiagramKind::EntityRelationship(data) = &mut diagram.kind {
            mutate(data);
        }
    });
}

fn entity<T: Default>(document: Document, index: usize, take: impl FnOnce(&Entity) -> T) -> T {
    with(document, |data| {
        data.entities.get(index).map(take).unwrap_or_default()
    })
}

fn rename(document: Document, index: usize, id: String) {
    change(document, move |data| {
        let Some(entry) = data.entities.get_mut(index) else {
            return;
        };
        let previous = std::mem::replace(&mut entry.id, id.clone());
        if previous == id {
            return;
        }
        for relationship in &mut data.relationships {
            if relationship.from == previous {
                relationship.from = id.clone();
            }
            if relationship.to == previous {
                relationship.to = id.clone();
            }
        }
    });
}

fn change_entity(document: Document, index: usize, mutate: impl FnOnce(&mut Entity)) {
    change(document, |data| {
        if let Some(entry) = data.entities.get_mut(index) {
            mutate(entry);
        }
    });
}

fn attribute<T: Default>(
    document: Document,
    index: usize,
    position: usize,
    take: impl FnOnce(&Attribute) -> T,
) -> T {
    entity(document, index, |entry| {
        entry.attributes.get(position).map(take).unwrap_or_default()
    })
}

fn change_attribute(
    document: Document,
    index: usize,
    position: usize,
    mutate: impl FnOnce(&mut Attribute),
) {
    change_entity(document, index, |entry| {
        if let Some(value) = entry.attributes.get_mut(position) {
            mutate(value);
        }
    });
}

fn relationship<T: Default>(
    document: Document,
    index: usize,
    take: impl FnOnce(&Relationship) -> T,
) -> T {
    with(document, |data| {
        data.relationships.get(index).map(take).unwrap_or_default()
    })
}

fn change_relationship(document: Document, index: usize, mutate: impl FnOnce(&mut Relationship)) {
    change(document, |data| {
        if let Some(entry) = data.relationships.get_mut(index) {
            mutate(entry);
        }
    });
}
