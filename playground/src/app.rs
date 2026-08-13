use graphiti::validate::{Severity, issues};
use leptos::prelude::*;
use wasm_bindgen::JsCast;

use crate::builder::Builder;
use crate::document::new_document;
use crate::download::{save_png, save_svg};
use crate::editor::Editor;
use crate::form::Choice;
use crate::options::BASE_THEMES;
use crate::preview::preview;
use crate::samples::{SAMPLES, source_for};

#[derive(Clone, Copy, PartialEq)]
enum Tab {
    Diagram,
    Json,
}

#[component]
pub fn App() -> impl IntoView {
    let source = RwSignal::new(SAMPLES[0].source.to_string());
    let document = new_document(source);
    let theme = RwSignal::new("light");
    let tab = RwSignal::new(Tab::Diagram);
    let search = RwSignal::new(String::new());
    let rendered = Memo::new(move |_| preview(&source.get(), theme.get()));
    let drawing = RwSignal::new(String::new());
    let problems = Memo::new(move |_| {
        document
            .parsed
            .with(|value| value.as_ref().map(issues).unwrap_or_default())
    });

    Effect::new(move |_| {
        if let Ok(current) = rendered.get() {
            drawing.set(current.svg);
        }
    });

    let on_sample = move |event: web_sys::Event| {
        if let Some(select) = event
            .target()
            .and_then(|target| target.dyn_into::<web_sys::HtmlSelectElement>().ok())
        {
            source.set(source_for(&select.value()).to_string());
            search.set(String::new());
        }
    };

    let broken = move || rendered.with(|value| value.is_err());
    let message = move || rendered.with(|value| value.as_ref().err().cloned());
    let stat = move |take: fn(&crate::preview::Preview) -> String| {
        rendered.with(|value| match value {
            Ok(current) => take(current),
            Err(_) => "\u{2014}".to_string(),
        })
    };

    let on_svg = move |_| {
        rendered.with(|value| {
            if let Ok(current) = value {
                save_svg(&current.svg, current.kind);
            }
        });
    };
    let on_png = move |_| {
        rendered.with(|value| {
            if let Ok(current) = value {
                save_png(&current.svg, current.width, current.height, current.kind);
            }
        });
    };

    view! {
        <div class="shell">
            <aside class="panel">
                <div class="panel-title">"graphiti"</div>
                <div class="panel-hint">
                    "Diagrams as data. Build the document top down or edit the JSON, and it redraws as you change it."
                </div>

                <div class="row">
                    <label class="field">
                        <span>"Example"</span>
                        <select on:change=on_sample>
                            {SAMPLES
                                .iter()
                                .map(|sample| {
                                    view! { <option value=sample.name>{sample.name}</option> }
                                })
                                .collect_view()}
                        </select>
                    </label>
                    <Choice
                        label="Base theme"
                        options=BASE_THEMES
                        value=Signal::derive(move || theme.get())
                        change=Callback::new(move |chosen: &'static str| theme.set(chosen))
                    />
                </div>

                <div class="tabs">
                    <button
                        class="tab"
                        class:active=move || tab.get() == Tab::Diagram
                        on:click=move |_| tab.set(Tab::Diagram)
                    >
                        "Diagram"
                    </button>
                    <button
                        class="tab"
                        class:active=move || tab.get() == Tab::Json
                        on:click=move |_| tab.set(Tab::Json)
                    >
                        "JSON"
                    </button>
                </div>

                <div class="panel-body">
                    {move || match tab.get() {
                        Tab::Diagram => {
                            view! { <Builder document=document search=search /> }.into_any()
                        }
                        Tab::Json => view! { <Editor source=source /> }.into_any(),
                    }}
                </div>

                <Show when=move || message().is_some()>
                    <div class="error">{move || message().unwrap_or_default()}</div>
                </Show>

                <Show when=move || !problems.with(|list| list.is_empty())>
                    <div class="issues">
                        {move || {
                            problems
                                .get()
                                .into_iter()
                                .map(|issue| {
                                    let class = if issue.severity == Severity::Error {
                                        "issue broken"
                                    } else {
                                        "issue"
                                    };
                                    view! { <div class=class>{issue.message}</div> }
                                })
                                .collect_view()
                        }}
                    </div>
                </Show>

                <div class="row">
                    <button class="action" prop:disabled=broken on:click=on_svg>
                        "Save SVG"
                    </button>
                    <button class="action" prop:disabled=broken on:click=on_png>
                        "Save PNG"
                    </button>
                </div>

                <div class="stats">
                    <div class="stat">
                        <span>"Canvas"</span>
                        <span>
                            {move || {
                                stat(|current| {
                                    format!("{:.0} x {:.0}", current.width, current.height)
                                })
                            }}
                        </span>
                    </div>
                    <div class="stat">
                        <span>"Shapes"</span>
                        <span>{move || stat(|current| current.shapes.to_string())}</span>
                    </div>
                    <div class="stat">
                        <span>"Labels"</span>
                        <span>{move || stat(|current| current.labels.to_string())}</span>
                    </div>
                </div>
            </aside>

            <div
                class="stage"
                class:stale=broken
                inner_html=move || drawing.get()
            ></div>
        </div>
    }
}
