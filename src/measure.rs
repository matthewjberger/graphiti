use nightshade::text::font_engine::FontEngine;
use nightshade::text::text_data::FontKind;
use std::collections::HashMap;

pub struct TextMeasurer {
    engine: FontEngine,
    cache: HashMap<(String, u32, bool), f32>,
}

pub fn new_measurer() -> TextMeasurer {
    TextMeasurer {
        engine: FontEngine::new(),
        cache: HashMap::new(),
    }
}

pub fn measure_text(
    measurer: &mut TextMeasurer,
    text: &str,
    font_size: f32,
    monospace: bool,
) -> f32 {
    if text.is_empty() {
        return 0.0;
    }
    let key = (text.to_string(), font_size.to_bits(), monospace);
    if let Some(width) = measurer.cache.get(&key) {
        return *width;
    }
    let buffer = measurer.engine.shape_buffer(
        text,
        font_size,
        font_size * 1.2,
        None,
        if monospace {
            FontKind::Mono
        } else {
            FontKind::Default
        },
    );
    let width = buffer
        .layout_runs()
        .map(|run| run.line_w)
        .fold(0.0f32, f32::max);
    measurer.cache.insert(key, width);
    width
}
