use leptos::prelude::*;
use wasm_bindgen::JsCast;

use crate::highlight::highlight;

#[component]
pub fn Editor(source: RwSignal<String>) -> impl IntoView {
    let overlay: NodeRef<leptos::html::Pre> = NodeRef::new();

    let follow = move |area: &web_sys::HtmlTextAreaElement| {
        if let Some(pre) = overlay.get_untracked() {
            pre.set_scroll_top(area.scroll_top());
            pre.set_scroll_left(area.scroll_left());
        }
    };

    let on_input = move |event: web_sys::Event| {
        if let Some(area) = event
            .target()
            .and_then(|target| target.dyn_into::<web_sys::HtmlTextAreaElement>().ok())
        {
            source.set(area.value());
            follow(&area);
        }
    };

    let on_scroll = move |event: web_sys::Event| {
        if let Some(area) = event
            .target()
            .and_then(|target| target.dyn_into::<web_sys::HtmlTextAreaElement>().ok())
        {
            follow(&area);
        }
    };

    let tokens = move || {
        let mut runs = highlight(&source.get());
        runs.push(("tok-plain", "\n".to_string()));
        runs.into_iter()
            .map(|(class, text)| view! { <span class=class>{text}</span> })
            .collect_view()
    };

    view! {
        <div class="editor">
            <pre class="highlight" node_ref=overlay aria-hidden="true">
                {tokens}
            </pre>
            <textarea
                class="source"
                spellcheck="false"
                on:input=on_input
                on:scroll=on_scroll
                prop:value=move || source.get()
            ></textarea>
        </div>
    }
}
