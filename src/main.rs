mod app;
mod configs;
mod header;
mod infra;
mod invoke;
mod mainpage;
mod reducer;
mod components {
    pub mod input_auto;
}

use app::App;

fn main() {
    yew::Renderer::<App>::new().render();
}
