use graphiti::schema::DiagramKind;
use graphiti::schema::sequence::{
    Divider, Fragment, FragmentBranch, Message, Note, Participant, Sequence, SequenceStep,
};
use leptos::prelude::*;

use crate::document::{Document, edit, read};
use crate::form::{Card, Check, Choice, Reference, Section, TextInput, Toggle, matches};
use crate::options::{
    ACCENTS, FRAGMENT_KINDS, MESSAGE_KINDS, NOTE_PLACEMENTS, PARTICIPANT_KINDS, STEP_KINDS,
};

#[derive(Clone, Copy, PartialEq)]
enum Hop {
    Step(usize),
    Branch(usize),
}

pub fn view(document: Document, search: RwSignal<String>) -> impl IntoView {
    view! {
        <Section
            title=Signal::derive(move || {
                format!("Participants ({})", with(document, |data| data.participants.len()))
            })
            add=Callback::new(move |_| {
                search.set(String::new());
                change(
                    document,
                    |data| {
                        let id = format!("actor{}", data.participants.len() + 1);
                        data.participants
                            .push(Participant {
                                label: id.clone(),
                                id,
                                ..Participant::default()
                            });
                    },
                )
            })
        >
            <For
                each=move || visible_participants(document, search)
                key=|index| *index
                children=move |index| participant_card(document, index)
            />
        </Section>

        {steps_section(document, Vec::new(), "Steps".to_string())}
    }
}

fn participant_card(document: Document, index: usize) -> impl IntoView {
    view! {
        <Card
            title=format!("Participant {}", index + 1)
            remove=Callback::new(move |_| {
                change(
                    document,
                    move |data| {
                        if index < data.participants.len() {
                            data.participants.remove(index);
                        }
                    },
                )
            })
        >
            <div class="grid">
                <TextInput
                    label="Id"
                    value=Signal::derive(move || {
                        participant(document, index, |entry| entry.id.clone())
                    })
                    change=Callback::new(move |text: String| rename(document, index, text))
                />
                <TextInput
                    label="Label"
                    value=Signal::derive(move || {
                        participant(document, index, |entry| entry.label.clone())
                    })
                    change=Callback::new(move |text: String| {
                        change_participant(document, index, move |entry| entry.label = text)
                    })
                />
                <Choice
                    label="Kind"
                    options=PARTICIPANT_KINDS
                    value=Signal::derive(move || participant(document, index, |entry| entry.kind))
                    change=Callback::new(move |value| {
                        change_participant(document, index, move |entry| entry.kind = value)
                    })
                />
                <Choice
                    label="Accent"
                    options=ACCENTS
                    value=Signal::derive(move || participant(document, index, |entry| entry.accent))
                    change=Callback::new(move |value| {
                        change_participant(document, index, move |entry| entry.accent = value)
                    })
                />
            </div>
        </Card>
    }
}

fn steps_section(document: Document, path: Vec<Hop>, label: String) -> AnyView {
    let stored = StoredValue::new(path);
    view! {
        <Section
            title=Signal::derive(move || {
                format!("{label} ({})", count(document, &stored.get_value()))
            })
            add=Callback::new(move |_| {
                change(
                    document,
                    move |data| {
                        if let Some(steps) = container_mut(&mut data.steps, &stored.get_value()) {
                            steps.push(SequenceStep::Message(Message::default()));
                        }
                    },
                )
            })
        >
            <For
                each=move || 0..count(document, &stored.get_value())
                key=|position| *position
                children=move |position| {
                    let shape = Memo::new(move |_| kind_of(document, &stored.get_value(), position));
                    view! {
                        <div class="step">
                            {move || step_card(document, stored, position, shape.get())}
                        </div>
                    }
                }
            />
        </Section>
    }
    .into_any()
}

fn step_card(
    document: Document,
    stored: StoredValue<Vec<Hop>>,
    position: usize,
    shape: &'static str,
) -> AnyView {
    let header = move || {
        view! {
            <Choice
                label="Step"
                options=STEP_KINDS
                value=Signal::derive(move || kind_of(document, &stored.get_value(), position))
                change=Callback::new(move |chosen: &'static str| {
                    set_kind(document, &stored.get_value(), position, chosen)
                })
            />
        }
    };
    let title = (position + 1).to_string();
    let remove = Callback::new(move |_| {
        change(document, move |data| {
            if let Some(steps) = container_mut(&mut data.steps, &stored.get_value())
                && position < steps.len()
            {
                steps.remove(position);
            }
        })
    });

    match shape {
        "message" => view! {
            <Card title=title remove=remove>
                <div class="grid">
                    {header()}
                    <Reference
                        label="From"
                        options=Signal::derive(move || ids(document))
                        value=Signal::derive(move || {
                            message(document, &stored.get_value(), position, |step| step.from.clone())
                        })
                        change=Callback::new(move |text: String| {
                            change_message(
                                document,
                                &stored.get_value(),
                                position,
                                move |step| step.from = text,
                            )
                        })
                    />
                    <Reference
                        label="To"
                        options=Signal::derive(move || ids(document))
                        value=Signal::derive(move || {
                            message(document, &stored.get_value(), position, |step| step.to.clone())
                        })
                        change=Callback::new(move |text: String| {
                            change_message(
                                document,
                                &stored.get_value(),
                                position,
                                move |step| step.to = text,
                            )
                        })
                    />
                    <TextInput
                        label="Label"
                        value=Signal::derive(move || {
                            message(
                                document,
                                &stored.get_value(),
                                position,
                                |step| step.label.clone(),
                            )
                        })
                        change=Callback::new(move |text: String| {
                            change_message(
                                document,
                                &stored.get_value(),
                                position,
                                move |step| step.label = text,
                            )
                        })
                    />
                    <Choice
                        label="Kind"
                        options=MESSAGE_KINDS
                        value=Signal::derive(move || {
                            message(document, &stored.get_value(), position, |step| step.kind)
                        })
                        change=Callback::new(move |value| {
                            change_message(
                                document,
                                &stored.get_value(),
                                position,
                                move |step| step.kind = value,
                            )
                        })
                    />
                    <Choice
                        label="Accent"
                        options=ACCENTS
                        value=Signal::derive(move || {
                            message(document, &stored.get_value(), position, |step| step.accent)
                        })
                        change=Callback::new(move |value| {
                            change_message(
                                document,
                                &stored.get_value(),
                                position,
                                move |step| step.accent = value,
                            )
                        })
                    />
                </div>
                <div class="row">
                    <Toggle
                        label="Activate"
                        value=Signal::derive(move || {
                            message(document, &stored.get_value(), position, |step| step.activate)
                        })
                        change=Callback::new(move |flag: bool| {
                            change_message(
                                document,
                                &stored.get_value(),
                                position,
                                move |step| step.activate = flag,
                            )
                        })
                    />
                    <Toggle
                        label="Deactivate"
                        value=Signal::derive(move || {
                            message(document, &stored.get_value(), position, |step| step.deactivate)
                        })
                        change=Callback::new(move |flag: bool| {
                            change_message(
                                document,
                                &stored.get_value(),
                                position,
                                move |step| step.deactivate = flag,
                            )
                        })
                    />
                </div>
            </Card>
        }
        .into_any(),
        "note" => view! {
            <Card title=title remove=remove>
                <div class="grid">
                    {header()}
                    <TextInput
                        label="Text"
                        value=Signal::derive(move || {
                            note(document, &stored.get_value(), position, |step| step.text.clone())
                        })
                        change=Callback::new(move |text: String| {
                            change_note(
                                document,
                                &stored.get_value(),
                                position,
                                move |step| step.text = text,
                            )
                        })
                    />
                    <Choice
                        label="Placement"
                        options=NOTE_PLACEMENTS
                        value=Signal::derive(move || {
                            note(document, &stored.get_value(), position, |step| step.placement)
                        })
                        change=Callback::new(move |value| {
                            change_note(
                                document,
                                &stored.get_value(),
                                position,
                                move |step| step.placement = value,
                            )
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
                                        note(
                                            document,
                                            &stored.get_value(),
                                            position,
                                            |step| step.over.contains(&checked),
                                        )
                                    })
                                    change=Callback::new(move |flag: bool| {
                                        let member = toggled.clone();
                                        change_note(
                                            document,
                                            &stored.get_value(),
                                            position,
                                            move |step| {
                                                step.over.retain(|entry| entry != &member);
                                                if flag {
                                                    step.over.push(member);
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
        .into_any(),
        "fragment" => {
            let mut inner = stored.get_value();
            inner.push(Hop::Step(position));
            view! {
                <Card title=title remove=remove>
                    <div class="grid">
                        {header()}
                        <Choice
                            label="Kind"
                            options=FRAGMENT_KINDS
                            value=Signal::derive(move || {
                                fragment(
                                    document,
                                    &stored.get_value(),
                                    position,
                                    |step| step.kind,
                                )
                            })
                            change=Callback::new(move |value| {
                                change_fragment(
                                    document,
                                    &stored.get_value(),
                                    position,
                                    move |step| step.kind = value,
                                )
                            })
                        />
                        <TextInput
                            label="Label"
                            value=Signal::derive(move || {
                                fragment(
                                    document,
                                    &stored.get_value(),
                                    position,
                                    |step| step.label.clone(),
                                )
                            })
                            change=Callback::new(move |text: String| {
                                change_fragment(
                                    document,
                                    &stored.get_value(),
                                    position,
                                    move |step| step.label = text,
                                )
                            })
                        />
                    </div>
                    {steps_section(document, inner, "Steps".to_string())}
                    <Section
                        title=Signal::derive(move || {
                            format!(
                                "Branches ({})",
                                fragment(
                                    document,
                                    &stored.get_value(),
                                    position,
                                    |step| step.branches.len(),
                                ),
                            )
                        })
                        add=Callback::new(move |_| {
                            change_fragment(
                                document,
                                &stored.get_value(),
                                position,
                                move |step| {
                                    step.branches
                                        .push(FragmentBranch {
                                            label: format!("branch{}", step.branches.len() + 1),
                                            ..FragmentBranch::default()
                                        });
                                },
                            )
                        })
                    >
                        <For
                            each=move || {
                                0
                                    ..fragment(
                                        document,
                                        &stored.get_value(),
                                        position,
                                        |step| step.branches.len(),
                                    )
                            }
                            key=|branch| *branch
                            children=move |branch| branch_card(document, stored, position, branch)
                        />
                    </Section>
                </Card>
            }
            .into_any()
        }
        "divider" => view! {
            <Card title=title remove=remove>
                <div class="grid">
                    {header()}
                    <TextInput
                        label="Label"
                        value=Signal::derive(move || {
                            divider(
                                document,
                                &stored.get_value(),
                                position,
                                |step| step.label.clone(),
                            )
                        })
                        change=Callback::new(move |text: String| {
                            change_divider(
                                document,
                                &stored.get_value(),
                                position,
                                move |step| step.label = text,
                            )
                        })
                    />
                </div>
            </Card>
        }
        .into_any(),
        _ => ().into_any(),
    }
}

fn branch_card(
    document: Document,
    stored: StoredValue<Vec<Hop>>,
    position: usize,
    branch: usize,
) -> AnyView {
    let mut inner = stored.get_value();
    inner.push(Hop::Step(position));
    inner.push(Hop::Branch(branch));
    view! {
        <Card
            title=format!("Branch {}", branch + 1)
            remove=Callback::new(move |_| {
                change_fragment(
                    document,
                    &stored.get_value(),
                    position,
                    move |step| {
                        if branch < step.branches.len() {
                            step.branches.remove(branch);
                        }
                    },
                )
            })
        >
            <TextInput
                label="Label"
                value=Signal::derive(move || {
                    fragment(
                        document,
                        &stored.get_value(),
                        position,
                        |step| {
                            step.branches
                                .get(branch)
                                .map(|entry| entry.label.clone())
                                .unwrap_or_default()
                        },
                    )
                })
                change=Callback::new(move |text: String| {
                    change_fragment(
                        document,
                        &stored.get_value(),
                        position,
                        move |step| {
                            if let Some(entry) = step.branches.get_mut(branch) {
                                entry.label = text;
                            }
                        },
                    )
                })
            />
            {steps_section(document, inner, "Steps".to_string())}
        </Card>
    }
    .into_any()
}

fn container_of<'a>(root: &'a [SequenceStep], path: &[Hop]) -> Option<&'a [SequenceStep]> {
    let mut current = root;
    let mut index = 0;
    while index < path.len() {
        let Hop::Step(position) = path[index] else {
            return None;
        };
        let SequenceStep::Fragment(found) = current.get(position)? else {
            return None;
        };
        index += 1;
        if let Some(Hop::Branch(branch)) = path.get(index) {
            current = found.branches.get(*branch)?.steps.as_slice();
            index += 1;
        } else {
            current = found.steps.as_slice();
        }
    }
    Some(current)
}

fn container_mut<'a>(
    root: &'a mut Vec<SequenceStep>,
    path: &[Hop],
) -> Option<&'a mut Vec<SequenceStep>> {
    let mut current = root;
    let mut index = 0;
    while index < path.len() {
        let Hop::Step(position) = path[index] else {
            return None;
        };
        let SequenceStep::Fragment(found) = current.get_mut(position)? else {
            return None;
        };
        index += 1;
        if let Some(Hop::Branch(branch)) = path.get(index) {
            current = &mut found.branches.get_mut(*branch)?.steps;
            index += 1;
        } else {
            current = &mut found.steps;
        }
    }
    Some(current)
}

fn count(document: Document, path: &[Hop]) -> usize {
    with(document, |data| {
        container_of(&data.steps, path)
            .map(|steps| steps.len())
            .unwrap_or(0)
    })
}

fn kind_of(document: Document, path: &[Hop], position: usize) -> &'static str {
    with(document, |data| {
        match container_of(&data.steps, path).and_then(|steps| steps.get(position)) {
            Some(SequenceStep::Message(_)) => "message",
            Some(SequenceStep::Note(_)) => "note",
            Some(SequenceStep::Fragment(_)) => "fragment",
            Some(SequenceStep::Divider(_)) => "divider",
            None => "",
        }
    })
}

fn set_kind(document: Document, path: &[Hop], position: usize, name: &str) {
    let replacement = match name {
        "note" => SequenceStep::Note(Note::default()),
        "fragment" => SequenceStep::Fragment(Fragment::default()),
        "divider" => SequenceStep::Divider(Divider::default()),
        _ => SequenceStep::Message(Message::default()),
    };
    change(document, |data| {
        if let Some(steps) = container_mut(&mut data.steps, path)
            && let Some(step) = steps.get_mut(position)
        {
            let carried = label_of(step);
            *step = replacement;
            set_label(step, carried);
        }
    });
}

fn label_of(step: &SequenceStep) -> String {
    match step {
        SequenceStep::Message(value) => value.label.clone(),
        SequenceStep::Note(value) => value.text.clone(),
        SequenceStep::Fragment(value) => value.label.clone(),
        SequenceStep::Divider(value) => value.label.clone(),
    }
}

fn set_label(step: &mut SequenceStep, label: String) {
    match step {
        SequenceStep::Message(value) => value.label = label,
        SequenceStep::Note(value) => value.text = label,
        SequenceStep::Fragment(value) => value.label = label,
        SequenceStep::Divider(value) => value.label = label,
    }
}

fn message<T: Default>(
    document: Document,
    path: &[Hop],
    position: usize,
    take: impl FnOnce(&Message) -> T,
) -> T {
    with(document, |data| {
        match container_of(&data.steps, path).and_then(|steps| steps.get(position)) {
            Some(SequenceStep::Message(value)) => take(value),
            _ => T::default(),
        }
    })
}

fn change_message(
    document: Document,
    path: &[Hop],
    position: usize,
    mutate: impl FnOnce(&mut Message),
) {
    change(document, |data| {
        if let Some(steps) = container_mut(&mut data.steps, path)
            && let Some(SequenceStep::Message(value)) = steps.get_mut(position)
        {
            mutate(value);
        }
    });
}

fn note<T: Default>(
    document: Document,
    path: &[Hop],
    position: usize,
    take: impl FnOnce(&Note) -> T,
) -> T {
    with(document, |data| {
        match container_of(&data.steps, path).and_then(|steps| steps.get(position)) {
            Some(SequenceStep::Note(value)) => take(value),
            _ => T::default(),
        }
    })
}

fn change_note(document: Document, path: &[Hop], position: usize, mutate: impl FnOnce(&mut Note)) {
    change(document, |data| {
        if let Some(steps) = container_mut(&mut data.steps, path)
            && let Some(SequenceStep::Note(value)) = steps.get_mut(position)
        {
            mutate(value);
        }
    });
}

fn fragment<T: Default>(
    document: Document,
    path: &[Hop],
    position: usize,
    take: impl FnOnce(&Fragment) -> T,
) -> T {
    with(document, |data| {
        match container_of(&data.steps, path).and_then(|steps| steps.get(position)) {
            Some(SequenceStep::Fragment(value)) => take(value),
            _ => T::default(),
        }
    })
}

fn change_fragment(
    document: Document,
    path: &[Hop],
    position: usize,
    mutate: impl FnOnce(&mut Fragment),
) {
    change(document, |data| {
        if let Some(steps) = container_mut(&mut data.steps, path)
            && let Some(SequenceStep::Fragment(value)) = steps.get_mut(position)
        {
            mutate(value);
        }
    });
}

fn divider<T: Default>(
    document: Document,
    path: &[Hop],
    position: usize,
    take: impl FnOnce(&Divider) -> T,
) -> T {
    with(document, |data| {
        match container_of(&data.steps, path).and_then(|steps| steps.get(position)) {
            Some(SequenceStep::Divider(value)) => take(value),
            _ => T::default(),
        }
    })
}

fn change_divider(
    document: Document,
    path: &[Hop],
    position: usize,
    mutate: impl FnOnce(&mut Divider),
) {
    change(document, |data| {
        if let Some(steps) = container_mut(&mut data.steps, path)
            && let Some(SequenceStep::Divider(value)) = steps.get_mut(position)
        {
            mutate(value);
        }
    });
}

fn ids(document: Document) -> Vec<String> {
    with(document, |data| {
        data.participants
            .iter()
            .map(|entry| entry.id.clone())
            .collect()
    })
}

fn visible_participants(document: Document, search: RwSignal<String>) -> Vec<usize> {
    let needle = search.get();
    with(document, |data| {
        data.participants
            .iter()
            .enumerate()
            .filter(|(_, entry)| matches(&needle, &[&entry.id, &entry.label]))
            .map(|(index, _)| index)
            .collect()
    })
}

fn with<T: Default>(document: Document, take: impl FnOnce(&Sequence) -> T) -> T {
    read(document, |kind| match kind {
        DiagramKind::Sequence(data) => take(data),
        _ => T::default(),
    })
}

fn change(document: Document, mutate: impl FnOnce(&mut Sequence)) {
    edit(document, |diagram| {
        if let DiagramKind::Sequence(data) = &mut diagram.kind {
            mutate(data);
        }
    });
}

fn participant<T: Default>(
    document: Document,
    index: usize,
    take: impl FnOnce(&Participant) -> T,
) -> T {
    with(document, |data| {
        data.participants.get(index).map(take).unwrap_or_default()
    })
}

fn rename(document: Document, index: usize, id: String) {
    change(document, move |data| {
        let Some(entry) = data.participants.get_mut(index) else {
            return;
        };
        let previous = std::mem::replace(&mut entry.id, id.clone());
        if previous == id {
            return;
        }
        retarget(&mut data.steps, &previous, &id);
    });
}

fn retarget(steps: &mut [SequenceStep], previous: &str, id: &str) {
    for step in steps {
        match step {
            SequenceStep::Message(value) => {
                if value.from == previous {
                    value.from = id.to_string();
                }
                if value.to == previous {
                    value.to = id.to_string();
                }
            }
            SequenceStep::Note(value) => {
                for over in &mut value.over {
                    if over == previous {
                        *over = id.to_string();
                    }
                }
            }
            SequenceStep::Fragment(value) => {
                retarget(&mut value.steps, previous, id);
                for branch in &mut value.branches {
                    retarget(&mut branch.steps, previous, id);
                }
            }
            SequenceStep::Divider(_) => {}
        }
    }
}

fn change_participant(document: Document, index: usize, mutate: impl FnOnce(&mut Participant)) {
    change(document, |data| {
        if let Some(entry) = data.participants.get_mut(index) {
            mutate(entry);
        }
    });
}
