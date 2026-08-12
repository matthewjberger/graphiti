use leptos::prelude::*;
use nightshade_leptos::Engine;
use protocol::Command;
use wasm_bindgen::JsCast;

use crate::app::Status;
use crate::samples::{SAMPLES, source_for};

#[component]
pub fn Editor(engine: Engine, status: Status, initial: &'static str) -> impl IntoView {
    let source = RwSignal::new(initial.to_string());
    let theme = RwSignal::new("light".to_string());

    let send = move || {
        engine.send(&Command::Render {
            source: source.get(),
            theme: theme.get(),
        });
    };

    let on_input = move |event: web_sys::Event| {
        if let Some(area) = event
            .target()
            .and_then(|target| target.dyn_into::<web_sys::HtmlTextAreaElement>().ok())
        {
            source.set(area.value());
        }
    };

    let on_sample = move |event: web_sys::Event| {
        if let Some(select) = event
            .target()
            .and_then(|target| target.dyn_into::<web_sys::HtmlSelectElement>().ok())
        {
            source.set(source_for(&select.value()).to_string());
            send();
        }
    };

    let on_theme = move |event: web_sys::Event| {
        if let Some(select) = event
            .target()
            .and_then(|target| target.dyn_into::<web_sys::HtmlSelectElement>().ok())
        {
            theme.set(select.value());
            send();
        }
    };

    view! {
        <aside class="panel">
            <div class="panel-title">"graphiti"</div>
            <div class="panel-hint">"Diagrams as data. Edit the document and render it. A style block in the document overrides the base theme."</div>

            <div class="row">
                <label class="field">
                    <span>"Example"</span>
                    <select on:change=on_sample>
                        {SAMPLES
                            .iter()
                            .map(|sample| view! { <option value=sample.name>{sample.name}</option> })
                            .collect_view()}
                    </select>
                </label>
                <label class="field">
                    <span>"Base theme"</span>
                    <select on:change=on_theme>
                        <option value="light">"Light"</option>
                        <option value="dark">"Dark"</option>
                        <option value="mono">"Mono"</option>
                    </select>
                </label>
            </div>

            <textarea
                class="source"
                spellcheck="false"
                on:input=on_input
                prop:value=move || source.get()
            ></textarea>

            <button class="render" on:click=move |_| send()>
                "Render"
            </button>

            <Show when=move || status.error.get().is_some()>
                <div class="error">{move || status.error.get().unwrap_or_default()}</div>
            </Show>

            <div class="stats">
                <div class="stat">
                    <span>"Canvas"</span>
                    <span>
                        {move || {
                            format!("{:.0} x {:.0}", status.width.get(), status.height.get())
                        }}
                    </span>
                </div>
                <div class="stat">
                    <span>"Shapes"</span>
                    <span>{move || status.shapes.get()}</span>
                </div>
                <div class="stat">
                    <span>"Labels"</span>
                    <span>{move || status.labels.get()}</span>
                </div>
                <div class="stat">
                    <span>"Adapter"</span>
                    <span>{move || engine.state.adapter.get()}</span>
                </div>
            </div>
        </aside>
    }
}
