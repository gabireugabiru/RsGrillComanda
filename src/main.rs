mod app;
mod configs;
mod header;
mod infra;
mod invoke;
mod mainpage;
mod reducer;
mod components {
    pub mod input_auto;
    pub mod select;
}

use app::App;

fn main() {
    yew::Renderer::<App>::new().render();
}
