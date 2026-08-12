#[cfg(target_arch = "wasm32")]
fn main() {
    console_error_panic_hook::set_once();
    leptos::prelude::mount_to_body(playground::App);
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    eprintln!("the playground is a web app. serve it with `just playground`.");
    eprintln!("to render a diagram to a file instead, run: graphiti <input.json>");
    std::process::exit(1);
}
