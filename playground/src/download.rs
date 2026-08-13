use std::cell::RefCell;
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;

const PNG_SCALE: f32 = 2.0;

thread_local! {
    static PENDING: RefCell<Option<Closure<dyn FnMut()>>> = const { RefCell::new(None) };
}

pub fn save_svg(svg: &str, name: &str) {
    let Some(document) = document() else {
        return;
    };
    save(&document, &svg_url(svg), &format!("{name}.svg"));
}

pub fn save_png(svg: &str, width: f32, height: f32, name: &str) {
    let Some(document) = document() else {
        return;
    };
    let Ok(image) = web_sys::HtmlImageElement::new() else {
        return;
    };
    let target_width = (width * PNG_SCALE).round().max(1.0) as u32;
    let target_height = (height * PNG_SCALE).round().max(1.0) as u32;
    let file = format!("{name}.png");

    let held = image.clone();
    let ready = Closure::<dyn FnMut()>::new(move || {
        if let Some(url) = rasterize(&document, &held, target_width, target_height) {
            save(&document, &url, &file);
        }
    });
    image.set_onload(Some(ready.as_ref().unchecked_ref()));
    PENDING.with_borrow_mut(|slot| *slot = Some(ready));
    image.set_src(&svg_url(svg));
}

fn rasterize(
    document: &web_sys::Document,
    image: &web_sys::HtmlImageElement,
    width: u32,
    height: u32,
) -> Option<String> {
    let canvas = document
        .create_element("canvas")
        .ok()?
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .ok()?;
    canvas.set_width(width);
    canvas.set_height(height);
    let context = canvas
        .get_context("2d")
        .ok()??
        .dyn_into::<web_sys::CanvasRenderingContext2d>()
        .ok()?;
    context
        .draw_image_with_html_image_element_and_dw_and_dh(
            image,
            0.0,
            0.0,
            width as f64,
            height as f64,
        )
        .ok()?;
    canvas.to_data_url_with_type("image/png").ok()
}

fn save(document: &web_sys::Document, url: &str, file: &str) {
    let Some(anchor) = document
        .create_element("a")
        .ok()
        .and_then(|element| element.dyn_into::<web_sys::HtmlAnchorElement>().ok())
    else {
        return;
    };
    anchor.set_href(url);
    anchor.set_download(file);
    if let Some(body) = document.body() {
        let _ = body.append_child(&anchor);
        anchor.click();
        let _ = body.remove_child(&anchor);
    }
}

fn svg_url(svg: &str) -> String {
    format!(
        "data:image/svg+xml;charset=utf-8,{}",
        String::from(js_sys::encode_uri_component(svg))
    )
}

fn document() -> Option<web_sys::Document> {
    web_sys::window().and_then(|window| window.document())
}
