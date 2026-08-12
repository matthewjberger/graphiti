use clap::Parser;
use graphiti::render::populate_world;
use graphiti::{scene_for, schema, theme};
use nightshade_api::prelude::render_image;
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(
    name = "graphiti",
    about = "Renders a diagram described in JSON to an image"
)]
struct Arguments {
    input: PathBuf,
    #[arg(short, long)]
    output: Option<PathBuf>,
    #[arg(long, default_value = "light")]
    theme: String,
    #[arg(long, default_value_t = 2)]
    supersample: u32,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let arguments = Arguments::parse();
    let source = std::fs::read_to_string(&arguments.input)?;
    let diagram = schema::parse(&source)?;
    let selected = theme::theme_by_name(&arguments.theme).ok_or_else(|| {
        format!(
            "unknown theme '{}', expected one of {}",
            arguments.theme,
            theme::theme_names().join(", ")
        )
    })?;
    let effective_theme = schema::style(&diagram.kind)
        .theme
        .clone()
        .unwrap_or_else(|| arguments.theme.clone());

    let scene = scene_for(&diagram, &selected);
    let width = scene.size.x.ceil().max(64.0) as u32;
    let height = scene.size.y.ceil().max(64.0) as u32;
    let supersample = arguments.supersample.clamp(1, 4);

    let output = arguments
        .output
        .unwrap_or_else(|| arguments.input.with_extension("png"));
    if let Some(parent) = output.parent()
        && !parent.as_os_str().is_empty()
    {
        std::fs::create_dir_all(parent)?;
    }

    let render_target = if supersample > 1 {
        output.with_extension("supersampled.png")
    } else {
        output.clone()
    };

    render_image(
        width * supersample,
        height * supersample,
        render_target.clone(),
        move |world| {
            populate_world(world, &scene, supersample as f32);
        },
    );

    if supersample > 1 {
        downsample(&render_target, &output, width, height)?;
        let _ = std::fs::remove_file(&render_target);
    }

    println!(
        "wrote {} ({}x{}, {} theme)",
        output.display(),
        width,
        height,
        effective_theme
    );
    Ok(())
}

fn downsample(
    source: &Path,
    destination: &Path,
    width: u32,
    height: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    let image = image::open(source)?;
    let resized = image.resize_exact(width, height, image::imageops::FilterType::Lanczos3);
    resized.save(destination)?;
    Ok(())
}
