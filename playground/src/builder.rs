mod class_diagram;
mod entity_relationship;
mod flowchart;
mod sequence;
mod state_diagram;
mod style;

use graphiti::schema;
use leptos::prelude::*;

use crate::document::{Document, edit, kind_name, read, set_kind};
use crate::form::{Choice, TextInput};
use crate::options::KINDS;
use style::StyleBlock;

#[component]
pub fn Builder(document: Document, search: RwSignal<String>) -> impl IntoView {
    view! {
        <div class="builder">
            <div class="row">
                <Choice
                    label="Kind"
                    options=KINDS
                    value=Signal::derive(move || kind_name(document))
                    change=Callback::new(move |name: &'static str| set_kind(document, name))
                />
                <TextInput
                    label="Title"
                    value=Signal::derive(move || {
                        read(document, |kind| {
                            schema::title(kind).unwrap_or_default().to_string()
                        })
                    })
                    change=Callback::new(move |text: String| {
                        edit(
                            document,
                            move |diagram| {
                                *schema::title_mut(&mut diagram.kind) = crate::form::optional(text);
                            },
                        )
                    })
                />
            </div>
            <TextInput
                label="Search"
                placeholder="filter by id or label"
                value=Signal::derive(move || search.get())
                change=Callback::new(move |text: String| search.set(text))
            />
            {move || match kind_name(document) {
                "flowchart" => flowchart::view(document, search).into_any(),
                "sequence" => sequence::view(document, search).into_any(),
                "class" => class_diagram::view(document, search).into_any(),
                "state" => state_diagram::view(document, search).into_any(),
                "entity_relationship" => {
                    entity_relationship::view(document, search).into_any()
                }
                _ => ().into_any(),
            }}
            <StyleBlock document=document />
        </div>
    }
}
