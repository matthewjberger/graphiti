use leptos::prelude::*;
use wasm_bindgen::JsCast;

#[component]
pub fn Section(
    title: Signal<String>,
    #[prop(optional)] add: Option<Callback<()>>,
    children: Children,
) -> impl IntoView {
    view! {
        <div class="section">
            <div class="section-head">
                <span>{move || title.get()}</span>
                {add.map(|add| {
                    view! {
                        <button class="mini" on:click=move |_| add.run(())>
                            "Add"
                        </button>
                    }
                })}
            </div>
            {children()}
        </div>
    }
}

#[component]
pub fn Card(title: String, remove: Callback<()>, children: Children) -> impl IntoView {
    view! {
        <div class="card">
            <div class="card-head">
                <span>{title}</span>
                <button class="mini danger" on:click=move |_| remove.run(())>
                    "Remove"
                </button>
            </div>
            <div class="card-body">{children()}</div>
        </div>
    }
}

#[component]
pub fn TextInput(
    label: &'static str,
    value: Signal<String>,
    change: Callback<String>,
    #[prop(optional)] placeholder: &'static str,
) -> impl IntoView {
    let on_input = move |event: web_sys::Event| {
        if let Some(input) = input_of(&event) {
            change.run(input.value());
        }
    };
    view! {
        <label class="field">
            <span>{label}</span>
            <input
                type="text"
                spellcheck="false"
                placeholder=placeholder
                on:input=on_input
                prop:value=move || value.get()
            />
        </label>
    }
}

#[component]
pub fn TextArea(
    label: &'static str,
    value: Signal<String>,
    change: Callback<String>,
    commit: Callback<String>,
) -> impl IntoView {
    let text_of = |event: &web_sys::Event| {
        event
            .target()
            .and_then(|target| target.dyn_into::<web_sys::HtmlTextAreaElement>().ok())
            .map(|area| area.value())
    };
    let on_input = move |event: web_sys::Event| {
        if let Some(text) = text_of(&event) {
            change.run(text);
        }
    };
    let on_change = move |event: web_sys::Event| {
        if let Some(text) = text_of(&event) {
            commit.run(text);
        }
    };
    view! {
        <label class="field wide">
            <span>{label}</span>
            <textarea
                class="lines"
                spellcheck="false"
                on:input=on_input
                on:change=on_change
                prop:value=move || value.get()
            ></textarea>
        </label>
    }
}

#[component]
pub fn NumberInput(
    label: &'static str,
    value: Signal<Option<f32>>,
    change: Callback<Option<f32>>,
    #[prop(optional)] placeholder: &'static str,
) -> impl IntoView {
    let typed = RwSignal::new(None::<String>);
    let on_input = move |event: web_sys::Event| {
        if let Some(input) = input_of(&event) {
            let text = input.value();
            typed.set(Some(text.clone()));
            change.run(number(&text));
        }
    };
    let shown = move || {
        let current = value.get();
        match typed.get() {
            Some(raw) if number(&raw) == current => raw,
            _ => current.map(|value| value.to_string()).unwrap_or_default(),
        }
    };
    view! {
        <label class="field">
            <span>{label}</span>
            <input
                type="number"
                step="any"
                placeholder=placeholder
                on:input=on_input
                prop:value=shown
            />
        </label>
    }
}

fn number(text: &str) -> Option<f32> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        None
    } else {
        trimmed.parse::<f32>().ok()
    }
}

#[component]
pub fn Toggle(label: &'static str, value: Signal<bool>, change: Callback<bool>) -> impl IntoView {
    let on_change = move |event: web_sys::Event| {
        if let Some(input) = input_of(&event) {
            change.run(input.checked());
        }
    };
    view! {
        <label class="toggle">
            <input type="checkbox" on:change=on_change prop:checked=move || value.get() />
            <span>{label}</span>
        </label>
    }
}

#[component]
pub fn Check(label: String, value: Signal<bool>, change: Callback<bool>) -> impl IntoView {
    let on_change = move |event: web_sys::Event| {
        if let Some(input) = input_of(&event) {
            change.run(input.checked());
        }
    };
    view! {
        <label class="check">
            <input type="checkbox" on:change=on_change prop:checked=move || value.get() />
            <span>{label}</span>
        </label>
    }
}

#[component]
pub fn Choice<T>(
    label: &'static str,
    options: &'static [(&'static str, T)],
    value: Signal<T>,
    change: Callback<T>,
) -> impl IntoView
where
    T: Copy + PartialEq + Send + Sync + 'static,
{
    let on_change = move |event: web_sys::Event| {
        if let Some(select) = select_of(&event)
            && let Ok(position) = select.value().parse::<usize>()
            && let Some((_, chosen)) = options.get(position)
        {
            change.run(*chosen);
        }
    };
    view! {
        <label class="field">
            <span>{label}</span>
            <select on:change=on_change>
                {options
                    .iter()
                    .enumerate()
                    .map(|(position, (name, candidate))| {
                        let candidate = *candidate;
                        view! {
                            <option
                                value=position.to_string()
                                prop:selected=move || value.get() == candidate
                            >
                                {*name}
                            </option>
                        }
                    })
                    .collect_view()}
            </select>
        </label>
    }
}

#[component]
pub fn Reference(
    label: &'static str,
    options: Signal<Vec<String>>,
    value: Signal<String>,
    change: Callback<String>,
) -> impl IntoView {
    let choices = move || {
        let mut list: Vec<String> = Vec::new();
        for candidate in options.get() {
            if !list.contains(&candidate) {
                list.push(candidate);
            }
        }
        let current = value.get();
        if !list.contains(&current) {
            list.insert(0, current);
        }
        list
    };
    let on_change = move |event: web_sys::Event| {
        if let Some(select) = select_of(&event) {
            change.run(select.value());
        }
    };
    view! {
        <label class="field">
            <span>{label}</span>
            <select on:change=on_change>
                <For
                    each=choices
                    key=|candidate| candidate.clone()
                    children=move |candidate| {
                        let shown = candidate.clone();
                        let compared = candidate.clone();
                        view! {
                            <option
                                value=candidate
                                prop:selected=move || value.get() == compared
                            >
                                {shown}
                            </option>
                        }
                    }
                />
            </select>
        </label>
    }
}

pub fn optional(text: String) -> Option<String> {
    if text.is_empty() { None } else { Some(text) }
}

pub fn matches(needle: &str, haystack: &[&str]) -> bool {
    if needle.is_empty() {
        return true;
    }
    let needle = needle.to_lowercase();
    haystack
        .iter()
        .any(|value| value.to_lowercase().contains(&needle))
}

fn input_of(event: &web_sys::Event) -> Option<web_sys::HtmlInputElement> {
    event
        .target()
        .and_then(|target| target.dyn_into::<web_sys::HtmlInputElement>().ok())
}

fn select_of(event: &web_sys::Event) -> Option<web_sys::HtmlSelectElement> {
    event
        .target()
        .and_then(|target| target.dyn_into::<web_sys::HtmlSelectElement>().ok())
}
