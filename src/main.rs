mod app;
mod backups;
mod configs;
mod header;
mod infra;
mod invoke;
mod mainpage;
mod reducer;
mod sort_dates;
mod stock;
mod stock_state;
mod components {
    pub mod input_auto;
    pub mod select;
}

use app::App;

fn main() {
    yew::Renderer::<App>::new().render();
}
