use graphiti::schema::DiagramKind;
use graphiti::schema::flowchart::{FlowEdge, FlowGroup, FlowNode, Flowchart};
use leptos::prelude::*;

use crate::document::{Document, edit, read};
use crate::form::{Card, Check, Choice, Reference, Section, TextInput, matches, optional};
use crate::options::{ACCENTS, ARROWS, DIRECTIONS, LINE_STYLES, ROUTINGS, SHAPES};

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
            <Choice
                label="Routing"
                options=ROUTINGS
                value=Signal::derive(move || with(document, |data| data.routing))
                change=Callback::new(move |value| {
                    change(document, move |data| data.routing = value)
                })
            />
        </div>

        <Section
            title=Signal::derive(move || {
                format!("Nodes ({})", with(document, |data| data.nodes.len()))
            })
            add=Callback::new(move |_| {
                change(document, |data| {
                    let id = format!("node{}", data.nodes.len() + 1);
                    data.nodes.push(FlowNode { label: id.clone(), id, ..FlowNode::default() });
                })
            })
        >
            <For
                each=move || visible_nodes(document, search)
                key=|index| *index
                children=move |index| node_card(document, index)
            />
        </Section>

        <Section
            title=Signal::derive(move || {
                format!("Edges ({})", with(document, |data| data.edges.len()))
            })
            add=Callback::new(move |_| {
                change(document, |data| {
                    let first = data.nodes.first().map(|node| node.id.clone()).unwrap_or_default();
                    data.edges.push(FlowEdge { from: first.clone(), to: first, ..FlowEdge::default() });
                })
            })
        >
            <For
                each=move || visible_edges(document, search)
                key=|index| *index
                children=move |index| edge_card(document, index, identifiers)
            />
        </Section>

        <Section
            title=Signal::derive(move || {
                format!("Groups ({})", with(document, |data| data.groups.len()))
            })
            add=Callback::new(move |_| {
                change(document, |data| {
                    let id = format!("group{}", data.groups.len() + 1);
                    data.groups.push(FlowGroup { label: id.clone(), id, ..FlowGroup::default() });
                })
            })
        >
            <For
                each=move || visible_groups(document, search)
                key=|index| *index
                children=move |index| group_card(document, index)
            />
        </Section>
    }
}

fn node_card(document: Document, index: usize) -> impl IntoView {
    view! {
        <Card
            title=format!("Node {}", index + 1)
            remove=Callback::new(move |_| {
                change(document, move |data| {
                    if index < data.nodes.len() {
                        data.nodes.remove(index);
                    }
                })
            })
        >
            <div class="grid">
                <TextInput
                    label="Id"
                    value=Signal::derive(move || node(document, index, |node| node.id.clone()))
                    change=Callback::new(move |text: String| rename(document, index, text))
                />
                <TextInput
                    label="Label"
                    value=Signal::derive(move || node(document, index, |node| node.label.clone()))
                    change=Callback::new(move |text: String| {
                        change_node(document, index, move |node| node.label = text)
                    })
                />
                <Choice
                    label="Shape"
                    options=SHAPES
                    value=Signal::derive(move || node(document, index, |node| node.shape))
                    change=Callback::new(move |value| {
                        change_node(document, index, move |node| node.shape = value)
                    })
                />
                <Choice
                    label="Accent"
                    options=ACCENTS
                    value=Signal::derive(move || node(document, index, |node| node.accent))
                    change=Callback::new(move |value| {
                        change_node(document, index, move |node| node.accent = value)
                    })
                />
                <TextInput
                    label="Detail"
                    value=Signal::derive(move || {
                        node(document, index, |node| node.detail.clone().unwrap_or_default())
                    })
                    change=Callback::new(move |text: String| {
                        change_node(document, index, move |node| node.detail = optional(text))
                    })
                />
            </div>
        </Card>
    }
}

fn edge_card(document: Document, index: usize, identifiers: Signal<Vec<String>>) -> impl IntoView {
    view! {
        <Card
            title=format!("Edge {}", index + 1)
            remove=Callback::new(move |_| {
                change(document, move |data| {
                    if index < data.edges.len() {
                        data.edges.remove(index);
                    }
                })
            })
        >
            <div class="grid">
                <Reference
                    label="From"
                    options=identifiers
                    value=Signal::derive(move || edge(document, index, |edge| edge.from.clone()))
                    change=Callback::new(move |text: String| {
                        change_edge(document, index, move |edge| edge.from = text)
                    })
                />
                <Reference
                    label="To"
                    options=identifiers
                    value=Signal::derive(move || edge(document, index, |edge| edge.to.clone()))
                    change=Callback::new(move |text: String| {
                        change_edge(document, index, move |edge| edge.to = text)
                    })
                />
                <TextInput
                    label="Label"
                    value=Signal::derive(move || {
                        edge(document, index, |edge| edge.label.clone().unwrap_or_default())
                    })
                    change=Callback::new(move |text: String| {
                        change_edge(document, index, move |edge| edge.label = optional(text))
                    })
                />
                <Choice
                    label="Line"
                    options=LINE_STYLES
                    value=Signal::derive(move || edge(document, index, |edge| edge.style))
                    change=Callback::new(move |value| {
                        change_edge(document, index, move |edge| edge.style = value)
                    })
                />
                <Choice
                    label="Head"
                    options=ARROWS
                    value=Signal::derive(move || edge(document, index, |edge| edge.head))
                    change=Callback::new(move |value| {
                        change_edge(document, index, move |edge| edge.head = value)
                    })
                />
                <Choice
                    label="Tail"
                    options=ARROWS
                    value=Signal::derive(move || edge(document, index, |edge| edge.tail))
                    change=Callback::new(move |value| {
                        change_edge(document, index, move |edge| edge.tail = value)
                    })
                />
                <Choice
                    label="Accent"
                    options=ACCENTS
                    value=Signal::derive(move || edge(document, index, |edge| edge.accent))
                    change=Callback::new(move |value| {
                        change_edge(document, index, move |edge| edge.accent = value)
                    })
                />
            </div>
        </Card>
    }
}

fn group_card(document: Document, index: usize) -> impl IntoView {
    view! {
        <Card
            title=format!("Group {}", index + 1)
            remove=Callback::new(move |_| {
                change(document, move |data| {
                    if index < data.groups.len() {
                        data.groups.remove(index);
                    }
                })
            })
        >
            <div class="grid">
                <TextInput
                    label="Id"
                    value=Signal::derive(move || group(document, index, |group| group.id.clone()))
                    change=Callback::new(move |text: String| {
                        change_group(document, index, move |group| group.id = text)
                    })
                />
                <TextInput
                    label="Label"
                    value=Signal::derive(move || group(document, index, |group| group.label.clone()))
                    change=Callback::new(move |text: String| {
                        change_group(document, index, move |group| group.label = text)
                    })
                />
                <Choice
                    label="Accent"
                    options=ACCENTS
                    value=Signal::derive(move || group(document, index, |group| group.accent))
                    change=Callback::new(move |value| {
                        change_group(document, index, move |group| group.accent = value)
                    })
                />
            </div>
            <div class="members">
                <For
                    each=move || ids(document)
                    key=|id| id.clone()
                    children=move |id| {
                        let checked = id.clone();
                        let toggled = id.clone();
                        view! {
                            <Check
                                label=id
                                value=Signal::derive(move || {
                                    group(document, index, |group| group.nodes.contains(&checked))
                                })
                                change=Callback::new(move |value: bool| {
                                    let member = toggled.clone();
                                    change_group(
                                        document,
                                        index,
                                        move |group| {
                                            group.nodes.retain(|entry| entry != &member);
                                            if value {
                                                group.nodes.push(member);
                                            }
                                        },
                                    )
                                })
                            />
                        }
                    }
                />
            </div>
        </Card>
    }
}

fn ids(document: Document) -> Vec<String> {
    with(document, |data| {
        data.nodes.iter().map(|node| node.id.clone()).collect()
    })
}

fn visible_nodes(document: Document, search: RwSignal<String>) -> Vec<usize> {
    let needle = search.get();
    with(document, |data| {
        data.nodes
            .iter()
            .enumerate()
            .filter(|(_, node)| matches(&needle, &[&node.id, &node.label]))
            .map(|(index, _)| index)
            .collect()
    })
}

fn visible_edges(document: Document, search: RwSignal<String>) -> Vec<usize> {
    let needle = search.get();
    with(document, |data| {
        data.edges
            .iter()
            .enumerate()
            .filter(|(_, edge)| {
                matches(
                    &needle,
                    &[&edge.from, &edge.to, edge.label.as_deref().unwrap_or("")],
                )
            })
            .map(|(index, _)| index)
            .collect()
    })
}

fn visible_groups(document: Document, search: RwSignal<String>) -> Vec<usize> {
    let needle = search.get();
    with(document, |data| {
        data.groups
            .iter()
            .enumerate()
            .filter(|(_, group)| matches(&needle, &[&group.id, &group.label]))
            .map(|(index, _)| index)
            .collect()
    })
}

fn with<T: Default>(document: Document, take: impl FnOnce(&Flowchart) -> T) -> T {
    read(document, |kind| match kind {
        DiagramKind::Flowchart(data) => take(data),
        _ => T::default(),
    })
}

fn change(document: Document, mutate: impl FnOnce(&mut Flowchart)) {
    edit(document, |diagram| {
        if let DiagramKind::Flowchart(data) = &mut diagram.kind {
            mutate(data);
        }
    });
}

fn node<T: Default>(document: Document, index: usize, take: impl FnOnce(&FlowNode) -> T) -> T {
    with(document, |data| {
        data.nodes.get(index).map(take).unwrap_or_default()
    })
}

fn rename(document: Document, index: usize, id: String) {
    change(document, move |data| {
        let Some(node) = data.nodes.get_mut(index) else {
            return;
        };
        let previous = std::mem::replace(&mut node.id, id.clone());
        if previous == id {
            return;
        }
        for edge in &mut data.edges {
            if edge.from == previous {
                edge.from = id.clone();
            }
            if edge.to == previous {
                edge.to = id.clone();
            }
        }
        for group in &mut data.groups {
            for member in &mut group.nodes {
                if *member == previous {
                    *member = id.clone();
                }
            }
        }
    });
}

fn change_node(document: Document, index: usize, mutate: impl FnOnce(&mut FlowNode)) {
    change(document, |data| {
        if let Some(node) = data.nodes.get_mut(index) {
            mutate(node);
        }
    });
}

fn edge<T: Default>(document: Document, index: usize, take: impl FnOnce(&FlowEdge) -> T) -> T {
    with(document, |data| {
        data.edges.get(index).map(take).unwrap_or_default()
    })
}

fn change_edge(document: Document, index: usize, mutate: impl FnOnce(&mut FlowEdge)) {
    change(document, |data| {
        if let Some(edge) = data.edges.get_mut(index) {
            mutate(edge);
        }
    });
}

fn group<T: Default>(document: Document, index: usize, take: impl FnOnce(&FlowGroup) -> T) -> T {
    with(document, |data| {
        data.groups.get(index).map(take).unwrap_or_default()
    })
}

fn change_group(document: Document, index: usize, mutate: impl FnOnce(&mut FlowGroup)) {
    change(document, |data| {
        if let Some(group) = data.groups.get_mut(index) {
            mutate(group);
        }
    });
}
