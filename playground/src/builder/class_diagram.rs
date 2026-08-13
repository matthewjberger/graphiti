use graphiti::schema::DiagramKind;
use graphiti::schema::class_diagram::{Class, ClassDiagram, ClassRelation, Member};
use leptos::prelude::*;

use crate::document::{Document, edit, read};
use crate::form::{Card, Choice, Reference, Section, TextInput, Toggle, matches, optional};
use crate::options::{ACCENTS, DIRECTIONS, RELATION_KINDS, VISIBILITIES};

#[derive(Clone, Copy, PartialEq)]
enum Group {
    Fields,
    Methods,
}

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
                format!("Classes ({})", with(document, |data| data.classes.len()))
            })
            add=Callback::new(move |_| {
                change(
                    document,
                    |data| {
                        let id = format!("class{}", data.classes.len() + 1);
                        data.classes.push(Class { name: id.clone(), id, ..Class::default() });
                    },
                )
            })
        >
            <For
                each=move || visible_classes(document, search)
                key=|index| *index
                children=move |index| class_card(document, index)
            />
        </Section>

        <Section
            title=Signal::derive(move || {
                format!("Relations ({})", with(document, |data| data.relations.len()))
            })
            add=Callback::new(move |_| {
                change(
                    document,
                    |data| {
                        let first = data
                            .classes
                            .first()
                            .map(|entry| entry.id.clone())
                            .unwrap_or_default();
                        data.relations
                            .push(ClassRelation {
                                from: first.clone(),
                                to: first,
                                ..ClassRelation::default()
                            });
                    },
                )
            })
        >
            <For
                each=move || visible_relations(document, search)
                key=|index| *index
                children=move |index| relation_card(document, index, identifiers)
            />
        </Section>
    }
}

fn class_card(document: Document, index: usize) -> impl IntoView {
    view! {
        <Card
            title=format!("Class {}", index + 1)
            remove=Callback::new(move |_| {
                change(
                    document,
                    move |data| {
                        if index < data.classes.len() {
                            data.classes.remove(index);
                        }
                    },
                )
            })
        >
            <div class="grid">
                <TextInput
                    label="Id"
                    value=Signal::derive(move || class(document, index, |entry| entry.id.clone()))
                    change=Callback::new(move |text: String| rename(document, index, text))
                />
                <TextInput
                    label="Name"
                    value=Signal::derive(move || class(document, index, |entry| entry.name.clone()))
                    change=Callback::new(move |text: String| {
                        change_class(document, index, move |entry| entry.name = text)
                    })
                />
                <TextInput
                    label="Stereotype"
                    placeholder="interface"
                    value=Signal::derive(move || {
                        class(document, index, |entry| entry.stereotype.clone().unwrap_or_default())
                    })
                    change=Callback::new(move |text: String| {
                        change_class(document, index, move |entry| entry.stereotype = optional(text))
                    })
                />
                <Choice
                    label="Accent"
                    options=ACCENTS
                    value=Signal::derive(move || class(document, index, |entry| entry.accent))
                    change=Callback::new(move |value| {
                        change_class(document, index, move |entry| entry.accent = value)
                    })
                />
            </div>
            {members_section(document, index, Group::Fields)}
            {members_section(document, index, Group::Methods)}
        </Card>
    }
}

fn members_section(document: Document, index: usize, group: Group) -> impl IntoView {
    view! {
        <Section
            title=Signal::derive(move || {
                let count = class(document, index, |entry| members_of(entry, group).len());
                match group {
                    Group::Fields => format!("Fields ({count})"),
                    Group::Methods => format!("Methods ({count})"),
                }
            })
            add=Callback::new(move |_| {
                change_class(
                    document,
                    index,
                    move |entry| {
                        let count = members_of(entry, group).len();
                        members_mut(entry, group)
                            .push(Member {
                                name: format!("member{}", count + 1),
                                ..Member::default()
                            });
                    },
                )
            })
        >
            <For
                each=move || 0..class(document, index, |entry| members_of(entry, group).len())
                key=|position| *position
                children=move |position| member_card(document, index, group, position)
            />
        </Section>
    }
}

fn member_card(document: Document, index: usize, group: Group, position: usize) -> impl IntoView {
    view! {
        <Card
            title=(position + 1).to_string()
            remove=Callback::new(move |_| {
                change_class(
                    document,
                    index,
                    move |entry| {
                        let members = members_mut(entry, group);
                        if position < members.len() {
                            members.remove(position);
                        }
                    },
                )
            })
        >
            <div class="grid">
                <TextInput
                    label="Name"
                    value=Signal::derive(move || {
                        member(document, index, group, position, |value| value.name.clone())
                    })
                    change=Callback::new(move |text: String| {
                        change_member(document, index, group, position, move |value| value.name = text)
                    })
                />
                <TextInput
                    label="Type"
                    value=Signal::derive(move || {
                        member(
                            document,
                            index,
                            group,
                            position,
                            |value| value.type_name.clone().unwrap_or_default(),
                        )
                    })
                    change=Callback::new(move |text: String| {
                        change_member(
                            document,
                            index,
                            group,
                            position,
                            move |value| value.type_name = optional(text),
                        )
                    })
                />
                <Choice
                    label="Visibility"
                    options=VISIBILITIES
                    value=Signal::derive(move || {
                        member(document, index, group, position, |value| value.visibility)
                    })
                    change=Callback::new(move |chosen| {
                        change_member(
                            document,
                            index,
                            group,
                            position,
                            move |value| value.visibility = chosen,
                        )
                    })
                />
            </div>
            <div class="row">
                <Toggle
                    label="Static"
                    value=Signal::derive(move || {
                        member(document, index, group, position, |value| value.is_static)
                    })
                    change=Callback::new(move |flag: bool| {
                        change_member(
                            document,
                            index,
                            group,
                            position,
                            move |value| value.is_static = flag,
                        )
                    })
                />
                <Toggle
                    label="Abstract"
                    value=Signal::derive(move || {
                        member(document, index, group, position, |value| value.is_abstract)
                    })
                    change=Callback::new(move |flag: bool| {
                        change_member(
                            document,
                            index,
                            group,
                            position,
                            move |value| value.is_abstract = flag,
                        )
                    })
                />
            </div>
        </Card>
    }
}

fn relation_card(
    document: Document,
    index: usize,
    identifiers: Signal<Vec<String>>,
) -> impl IntoView {
    view! {
        <Card
            title=format!("Relation {}", index + 1)
            remove=Callback::new(move |_| {
                change(
                    document,
                    move |data| {
                        if index < data.relations.len() {
                            data.relations.remove(index);
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
                        relation(document, index, |entry| entry.from.clone())
                    })
                    change=Callback::new(move |text: String| {
                        change_relation(document, index, move |entry| entry.from = text)
                    })
                />
                <Reference
                    label="To"
                    options=identifiers
                    value=Signal::derive(move || relation(document, index, |entry| entry.to.clone()))
                    change=Callback::new(move |text: String| {
                        change_relation(document, index, move |entry| entry.to = text)
                    })
                />
                <Choice
                    label="Kind"
                    options=RELATION_KINDS
                    value=Signal::derive(move || relation(document, index, |entry| entry.kind))
                    change=Callback::new(move |value| {
                        change_relation(document, index, move |entry| entry.kind = value)
                    })
                />
                <TextInput
                    label="Label"
                    value=Signal::derive(move || {
                        relation(document, index, |entry| entry.label.clone().unwrap_or_default())
                    })
                    change=Callback::new(move |text: String| {
                        change_relation(document, index, move |entry| entry.label = optional(text))
                    })
                />
                <TextInput
                    label="From cardinality"
                    placeholder="1"
                    value=Signal::derive(move || {
                        relation(
                            document,
                            index,
                            |entry| entry.from_cardinality.clone().unwrap_or_default(),
                        )
                    })
                    change=Callback::new(move |text: String| {
                        change_relation(
                            document,
                            index,
                            move |entry| entry.from_cardinality = optional(text),
                        )
                    })
                />
                <TextInput
                    label="To cardinality"
                    placeholder="0..*"
                    value=Signal::derive(move || {
                        relation(
                            document,
                            index,
                            |entry| entry.to_cardinality.clone().unwrap_or_default(),
                        )
                    })
                    change=Callback::new(move |text: String| {
                        change_relation(
                            document,
                            index,
                            move |entry| entry.to_cardinality = optional(text),
                        )
                    })
                />
            </div>
        </Card>
    }
}

fn members_of(class: &Class, group: Group) -> &Vec<Member> {
    match group {
        Group::Fields => &class.fields,
        Group::Methods => &class.methods,
    }
}

fn members_mut(class: &mut Class, group: Group) -> &mut Vec<Member> {
    match group {
        Group::Fields => &mut class.fields,
        Group::Methods => &mut class.methods,
    }
}

fn ids(document: Document) -> Vec<String> {
    with(document, |data| {
        data.classes.iter().map(|entry| entry.id.clone()).collect()
    })
}

fn visible_classes(document: Document, search: RwSignal<String>) -> Vec<usize> {
    let needle = search.get();
    with(document, |data| {
        data.classes
            .iter()
            .enumerate()
            .filter(|(_, entry)| matches(&needle, &[&entry.id, &entry.name]))
            .map(|(index, _)| index)
            .collect()
    })
}

fn visible_relations(document: Document, search: RwSignal<String>) -> Vec<usize> {
    let needle = search.get();
    with(document, |data| {
        data.relations
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

fn with<T: Default>(document: Document, take: impl FnOnce(&ClassDiagram) -> T) -> T {
    read(document, |kind| match kind {
        DiagramKind::Class(data) => take(data),
        _ => T::default(),
    })
}

fn change(document: Document, mutate: impl FnOnce(&mut ClassDiagram)) {
    edit(document, |diagram| {
        if let DiagramKind::Class(data) = &mut diagram.kind {
            mutate(data);
        }
    });
}

fn class<T: Default>(document: Document, index: usize, take: impl FnOnce(&Class) -> T) -> T {
    with(document, |data| {
        data.classes.get(index).map(take).unwrap_or_default()
    })
}

fn rename(document: Document, index: usize, id: String) {
    change(document, move |data| {
        let Some(entry) = data.classes.get_mut(index) else {
            return;
        };
        let previous = std::mem::replace(&mut entry.id, id.clone());
        if previous == id {
            return;
        }
        for relation in &mut data.relations {
            if relation.from == previous {
                relation.from = id.clone();
            }
            if relation.to == previous {
                relation.to = id.clone();
            }
        }
    });
}

fn change_class(document: Document, index: usize, mutate: impl FnOnce(&mut Class)) {
    change(document, |data| {
        if let Some(entry) = data.classes.get_mut(index) {
            mutate(entry);
        }
    });
}

fn member<T: Default>(
    document: Document,
    index: usize,
    group: Group,
    position: usize,
    take: impl FnOnce(&Member) -> T,
) -> T {
    class(document, index, |entry| {
        members_of(entry, group)
            .get(position)
            .map(take)
            .unwrap_or_default()
    })
}

fn change_member(
    document: Document,
    index: usize,
    group: Group,
    position: usize,
    mutate: impl FnOnce(&mut Member),
) {
    change_class(document, index, |entry| {
        if let Some(value) = members_mut(entry, group).get_mut(position) {
            mutate(value);
        }
    });
}

fn relation<T: Default>(
    document: Document,
    index: usize,
    take: impl FnOnce(&ClassRelation) -> T,
) -> T {
    with(document, |data| {
        data.relations.get(index).map(take).unwrap_or_default()
    })
}

fn change_relation(document: Document, index: usize, mutate: impl FnOnce(&mut ClassRelation)) {
    change(document, |data| {
        if let Some(entry) = data.relations.get_mut(index) {
            mutate(entry);
        }
    });
}
