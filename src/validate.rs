use crate::schema::sequence::{Message, Note, SequenceStep};
use crate::schema::{Diagram, DiagramKind};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Severity {
    Error,
    Warning,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Issue {
    pub severity: Severity,
    pub message: String,
}

pub fn issues(diagram: &Diagram) -> Vec<Issue> {
    let mut found = Vec::new();
    match &diagram.kind {
        DiagramKind::Flowchart(data) => {
            let ids: Vec<&str> = data.nodes.iter().map(|node| node.id.as_str()).collect();
            check_ids(&mut found, "node", &ids);
            if data.nodes.is_empty() {
                warn(&mut found, "This flowchart has no nodes yet.");
            }
            for (index, edge) in data.edges.iter().enumerate() {
                let owner = format!("edge {}", index + 1);
                check_reference(&mut found, &ids, &edge.from, &format!("{owner} source"));
                check_reference(&mut found, &ids, &edge.to, &format!("{owner} target"));
            }
            for group in &data.groups {
                for member in &group.nodes {
                    check_reference(&mut found, &ids, member, &format!("group '{}'", group.id));
                }
            }
        }
        DiagramKind::Sequence(data) => {
            let ids: Vec<&str> = data
                .participants
                .iter()
                .map(|entry| entry.id.as_str())
                .collect();
            check_ids(&mut found, "participant", &ids);
            if data.participants.is_empty() {
                warn(&mut found, "This sequence has no participants yet.");
            }
            let mut messages: Vec<&Message> = Vec::new();
            let mut notes: Vec<&Note> = Vec::new();
            walk(&data.steps, &mut messages, &mut notes);
            for (index, message) in messages.iter().enumerate() {
                let owner = format!("message {}", index + 1);
                check_reference(&mut found, &ids, &message.from, &format!("{owner} sender"));
                check_reference(&mut found, &ids, &message.to, &format!("{owner} receiver"));
            }
            for (index, note) in notes.iter().enumerate() {
                for over in &note.over {
                    check_reference(&mut found, &ids, over, &format!("note {}", index + 1));
                }
            }
        }
        DiagramKind::Class(data) => {
            let ids: Vec<&str> = data.classes.iter().map(|entry| entry.id.as_str()).collect();
            check_ids(&mut found, "class", &ids);
            if data.classes.is_empty() {
                warn(&mut found, "This class diagram has no classes yet.");
            }
            for (index, relation) in data.relations.iter().enumerate() {
                let owner = format!("relation {}", index + 1);
                check_reference(&mut found, &ids, &relation.from, &format!("{owner} source"));
                check_reference(&mut found, &ids, &relation.to, &format!("{owner} target"));
            }
        }
        DiagramKind::State(data) => {
            let ids: Vec<&str> = data.states.iter().map(|entry| entry.id.as_str()).collect();
            check_ids(&mut found, "state", &ids);
            if data.states.is_empty() {
                warn(&mut found, "This state diagram has no states yet.");
            }
            for (index, transition) in data.transitions.iter().enumerate() {
                let owner = format!("transition {}", index + 1);
                check_reference(
                    &mut found,
                    &ids,
                    &transition.from,
                    &format!("{owner} source"),
                );
                check_reference(&mut found, &ids, &transition.to, &format!("{owner} target"));
            }
        }
        DiagramKind::EntityRelationship(data) => {
            let ids: Vec<&str> = data
                .entities
                .iter()
                .map(|entry| entry.id.as_str())
                .collect();
            check_ids(&mut found, "entity", &ids);
            if data.entities.is_empty() {
                warn(&mut found, "This diagram has no entities yet.");
            }
            for (index, relationship) in data.relationships.iter().enumerate() {
                let owner = format!("relationship {}", index + 1);
                check_reference(
                    &mut found,
                    &ids,
                    &relationship.from,
                    &format!("{owner} source"),
                );
                check_reference(
                    &mut found,
                    &ids,
                    &relationship.to,
                    &format!("{owner} target"),
                );
            }
        }
    }
    found
}

fn walk<'a>(steps: &'a [SequenceStep], messages: &mut Vec<&'a Message>, notes: &mut Vec<&'a Note>) {
    for step in steps {
        match step {
            SequenceStep::Message(message) => messages.push(message),
            SequenceStep::Note(note) => notes.push(note),
            SequenceStep::Fragment(fragment) => {
                walk(&fragment.steps, messages, notes);
                for branch in &fragment.branches {
                    walk(&branch.steps, messages, notes);
                }
            }
            SequenceStep::Divider(_) => {}
        }
    }
}

fn check_ids(found: &mut Vec<Issue>, noun: &str, ids: &[&str]) {
    for (index, id) in ids.iter().enumerate() {
        if id.trim().is_empty() {
            error(found, format!("{noun} {} has a blank id.", index + 1));
            continue;
        }
        if ids[..index].contains(id) {
            error(found, format!("More than one {noun} uses the id '{id}'."));
        }
    }
}

fn check_reference(found: &mut Vec<Issue>, ids: &[&str], value: &str, owner: &str) {
    if value.trim().is_empty() {
        error(found, format!("{owner} is empty, so it will be dropped."));
    } else if !ids.contains(&value) {
        error(
            found,
            format!("{owner} points at '{value}', which does not exist, so it will be dropped."),
        );
    }
}

fn error(found: &mut Vec<Issue>, message: String) {
    found.push(Issue {
        severity: Severity::Error,
        message,
    });
}

fn warn(found: &mut Vec<Issue>, message: &str) {
    found.push(Issue {
        severity: Severity::Warning,
        message: message.to_string(),
    });
}
