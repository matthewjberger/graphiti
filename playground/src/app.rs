use leptos::prelude::*;
use nightshade_leptos::{EngineViewport, Loader, UiStyles, WebGpuGate, use_engine};
use protocol::Event;

use crate::editor::Editor;
use crate::samples::SAMPLES;

#[derive(Clone, Copy)]
pub struct Status {
    pub width: RwSignal<f32>,
    pub height: RwSignal<f32>,
    pub shapes: RwSignal<usize>,
    pub labels: RwSignal<usize>,
    pub error: RwSignal<Option<String>>,
}

#[component]
pub fn App() -> impl IntoView {
    view! {
        <UiStyles />
        <WebGpuGate>
            <Stage />
        </WebGpuGate>
    }
}

#[component]
fn Stage() -> impl IntoView {
    let engine = use_engine("runtime/worker.js");
    let status = Status {
        width: RwSignal::new(0.0),
        height: RwSignal::new(0.0),
        shapes: RwSignal::new(0),
        labels: RwSignal::new(0),
        error: RwSignal::new(None),
    };

    engine.on_custom(Callback::new(
        move |value: serde_json::Value| match serde_json::from_value::<Event>(value) {
            Ok(Event::Rendered {
                width,
                height,
                shapes,
                labels,
            }) => {
                status.width.set(width);
                status.height.set(height);
                status.shapes.set(shapes);
                status.labels.set(labels);
                status.error.set(None);
            }
            Ok(Event::Failed { message }) => status.error.set(Some(message)),
            Err(_) => {}
        },
    ));

    view! {
        <div class="shell">
            <Editor engine=engine status=status initial=SAMPLES[0].source />
            <div class="stage">
                <EngineViewport engine=engine />
                <Loader ready=engine.state.ready />
            </div>
        </div>
    }
}
