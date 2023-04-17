pub fn document() -> web_sys::Document {
    window().document().expect("Document no found in window")
}
pub fn window() -> web_sys::Window {
    web_sys::window().expect("Window not found")
}

macro_rules! log {
    ($($arg:tt)*) => {
        web_sys::console::log_1(&format!($($arg)*).into())
    };
    ($arg:literal) => {
        web_sys::console::log_1(&$arg.into())
    }
}

macro_rules! get {
    ($document:expr => $($arg:tt)*) => {

        match $document.get_element_by_id(&format!($($arg)*)) {
            Some(a) => {
                use wasm_bindgen::JsCast;
                a.dyn_into().ok()
            },
            None => None,
        }
    };
    ($document:expr => arg:literal) => {
        match $document.get_element_by_id(arg) {
            Some(a) => a.dyn_into().ok(),
            None => None,
        }
    };
}
macro_rules! alert {
    ($win:expr => $($arg:tt)*) => {
        if $win.alert_with_message(&format!($($arg)*)).is_err() {
            crate::infra::log!("Failed to alert")
        }
    };
    ($win:expr => $arg:literal) => {
        if $win.alert_with_message($arg).is_err() {
            crate::infra::log!("Failed to alert")
        }
    };
    ($win:expr) => {
        if $win.alert($arg).is_err() {
            crate::infra::log!("Failed to alert")
        }
    }
}

pub fn scroll_into_view<T>(a: T)
where
    web_sys::Element: From<T>,
{
    web_sys::Element::from(a)
        .scroll_into_view_with_scroll_into_view_options(dbg!(
            &web_sys::ScrollIntoViewOptions::new()
                .behavior(web_sys::ScrollBehavior::Instant)
                .block(web_sys::ScrollLogicalPosition::Center)
        ));
}

pub(crate) use alert;
pub(crate) use get;
pub(crate) use log;
