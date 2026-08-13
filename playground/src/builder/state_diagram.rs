use graphiti::schema::DiagramKind;
use graphiti::schema::state_diagram::{State, StateDiagram, Transition};
use leptos::prelude::*;

use crate::document::{Document, edit, read};
use crate::form::{Card, Choice, Reference, Section, TextArea, TextInput, matches, optional};
use crate::options::{ACCENTS, DIRECTIONS, LINE_STYLES, STATE_KINDS};

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
                format!("States ({})", with(document, |data| data.states.len()))
            })
            add=Callback::new(move |_| {
                change(
                    document,
                    |data| {
                        let id = format!("state{}", data.states.len() + 1);
                        data.states.push(State { label: id.clone(), id, ..State::default() });
                    },
                )
            })
        >
            <For
                each=move || visible_states(document, search)
                key=|index| *index
                children=move |index| state_card(document, index)
            />
        </Section>

        <Section
            title=Signal::derive(move || {
                format!("Transitions ({})", with(document, |data| data.transitions.len()))
            })
            add=Callback::new(move |_| {
                change(
                    document,
                    |data| {
                        let first = data
                            .states
                            .first()
                            .map(|state| state.id.clone())
                            .unwrap_or_default();
                        data.transitions
                            .push(Transition {
                                from: first.clone(),
                                to: first,
                                ..Transition::default()
                            });
                    },
                )
            })
        >
            <For
                each=move || visible_transitions(document, search)
                key=|index| *index
                children=move |index| transition_card(document, index, identifiers)
            />
        </Section>
    }
}

fn state_card(document: Document, index: usize) -> impl IntoView {
    view! {
        <Card
            title=format!("State {}", index + 1)
            remove=Callback::new(move |_| {
                change(
                    document,
                    move |data| {
                        if index < data.states.len() {
                            data.states.remove(index);
                        }
                    },
                )
            })
        >
            <div class="grid">
                <TextInput
                    label="Id"
                    value=Signal::derive(move || state(document, index, |state| state.id.clone()))
                    change=Callback::new(move |text: String| rename(document, index, text))
                />
                <TextInput
                    label="Label"
                    value=Signal::derive(move || state(document, index, |state| state.label.clone()))
                    change=Callback::new(move |text: String| {
                        change_state(document, index, move |state| state.label = text)
                    })
                />
                <Choice
                    label="Kind"
                    options=STATE_KINDS
                    value=Signal::derive(move || state(document, index, |state| state.kind))
                    change=Callback::new(move |value| {
                        change_state(document, index, move |state| state.kind = value)
                    })
                />
                <Choice
                    label="Accent"
                    options=ACCENTS
                    value=Signal::derive(move || state(document, index, |state| state.accent))
                    change=Callback::new(move |value| {
                        change_state(document, index, move |state| state.accent = value)
                    })
                />
            </div>
            <TextArea
                label="Description, one line each"
                value=Signal::derive(move || {
                    state(document, index, |state| state.description.join("\n"))
                })
                change=Callback::new(move |text: String| {
                    change_state(
                        document,
                        index,
                        move |state| {
                            state.description = text.lines().map(str::to_string).collect();
                        },
                    )
                })
            />
        </Card>
    }
}

fn transition_card(
    document: Document,
    index: usize,
    identifiers: Signal<Vec<String>>,
) -> impl IntoView {
    view! {
        <Card
            title=format!("Transition {}", index + 1)
            remove=Callback::new(move |_| {
                change(
                    document,
                    move |data| {
                        if index < data.transitions.len() {
                            data.transitions.remove(index);
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
                        transition(document, index, |transition| transition.from.clone())
                    })
                    change=Callback::new(move |text: String| {
                        change_transition(document, index, move |transition| transition.from = text)
                    })
                />
                <Reference
                    label="To"
                    options=identifiers
                    value=Signal::derive(move || {
                        transition(document, index, |transition| transition.to.clone())
                    })
                    change=Callback::new(move |text: String| {
                        change_transition(document, index, move |transition| transition.to = text)
                    })
                />
                <TextInput
                    label="Label"
                    value=Signal::derive(move || {
                        transition(
                            document,
                            index,
                            |transition| transition.label.clone().unwrap_or_default(),
                        )
                    })
                    change=Callback::new(move |text: String| {
                        change_transition(
                            document,
                            index,
                            move |transition| transition.label = optional(text),
                        )
                    })
                />
                <Choice
                    label="Line"
                    options=LINE_STYLES
                    value=Signal::derive(move || {
                        transition(document, index, |transition| transition.style)
                    })
                    change=Callback::new(move |value| {
                        change_transition(document, index, move |transition| transition.style = value)
                    })
                />
            </div>
        </Card>
    }
}

fn ids(document: Document) -> Vec<String> {
    with(document, |data| {
        data.states.iter().map(|state| state.id.clone()).collect()
    })
}

fn visible_states(document: Document, search: RwSignal<String>) -> Vec<usize> {
    let needle = search.get();
    with(document, |data| {
        data.states
            .iter()
            .enumerate()
            .filter(|(_, state)| matches(&needle, &[&state.id, &state.label]))
            .map(|(index, _)| index)
            .collect()
    })
}

fn visible_transitions(document: Document, search: RwSignal<String>) -> Vec<usize> {
    let needle = search.get();
    with(document, |data| {
        data.transitions
            .iter()
            .enumerate()
            .filter(|(_, transition)| {
                matches(
                    &needle,
                    &[
                        &transition.from,
                        &transition.to,
                        transition.label.as_deref().unwrap_or(""),
                    ],
                )
            })
            .map(|(index, _)| index)
            .collect()
    })
}

fn with<T: Default>(document: Document, take: impl FnOnce(&StateDiagram) -> T) -> T {
    read(document, |kind| match kind {
        DiagramKind::State(data) => take(data),
        _ => T::default(),
    })
}

fn change(document: Document, mutate: impl FnOnce(&mut StateDiagram)) {
    edit(document, |diagram| {
        if let DiagramKind::State(data) = &mut diagram.kind {
            mutate(data);
        }
    });
}

fn state<T: Default>(document: Document, index: usize, take: impl FnOnce(&State) -> T) -> T {
    with(document, |data| {
        data.states.get(index).map(take).unwrap_or_default()
    })
}

fn rename(document: Document, index: usize, id: String) {
    change(document, move |data| {
        let Some(state) = data.states.get_mut(index) else {
            return;
        };
        let previous = std::mem::replace(&mut state.id, id.clone());
        if previous == id {
            return;
        }
        for transition in &mut data.transitions {
            if transition.from == previous {
                transition.from = id.clone();
            }
            if transition.to == previous {
                transition.to = id.clone();
            }
        }
    });
}

fn change_state(document: Document, index: usize, mutate: impl FnOnce(&mut State)) {
    change(document, |data| {
        if let Some(state) = data.states.get_mut(index) {
            mutate(state);
        }
    });
}

fn transition<T: Default>(
    document: Document,
    index: usize,
    take: impl FnOnce(&Transition) -> T,
) -> T {
    with(document, |data| {
        data.transitions.get(index).map(take).unwrap_or_default()
    })
}

fn change_transition(document: Document, index: usize, mutate: impl FnOnce(&mut Transition)) {
    change(document, |data| {
        if let Some(transition) = data.transitions.get_mut(index) {
            mutate(transition);
        }
    });
}
