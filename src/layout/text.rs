use nalgebra_glm::{Vec2, vec2};

pub type Measure<'a> = &'a mut dyn FnMut(&str, f32) -> f32;

#[derive(Clone, Debug, Default)]
pub struct TextBlock {
    pub lines: Vec<String>,
    pub size: Vec2,
}

pub fn measure_block(
    text: &str,
    font_size: f32,
    line_height: f32,
    max_width: f32,
    measure: Measure,
) -> TextBlock {
    let mut lines = Vec::new();
    for paragraph in text.split('\n') {
        let trimmed = paragraph.trim();
        if trimmed.is_empty() {
            lines.push(String::new());
            continue;
        }
        wrap_paragraph(trimmed, font_size, max_width, measure, &mut lines);
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    let width = lines
        .iter()
        .map(|line| measure(line, font_size))
        .fold(0.0f32, f32::max);
    let height = lines.len() as f32 * font_size * line_height;
    TextBlock {
        lines,
        size: vec2(width, height),
    }
}

fn wrap_paragraph(
    paragraph: &str,
    font_size: f32,
    max_width: f32,
    measure: Measure,
    output: &mut Vec<String>,
) {
    let mut current = String::new();
    for word in paragraph.split_whitespace() {
        let candidate = if current.is_empty() {
            word.to_string()
        } else {
            format!("{current} {word}")
        };
        if measure(&candidate, font_size) <= max_width || current.is_empty() {
            current = candidate;
        } else {
            output.push(current);
            current = word.to_string();
        }
    }
    if !current.is_empty() {
        output.push(current);
    }
}

pub fn line_offsets(block: &TextBlock, font_size: f32, line_height: f32) -> Vec<f32> {
    let step = font_size * line_height;
    (0..block.lines.len())
        .map(|index| index as f32 * step + step * 0.5)
        .collect()
}
