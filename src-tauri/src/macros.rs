macro_rules! parsed_output {
    ($t:ty, $block:block) => {{
        let result: Result<$t, shared::errors::InError> = (move || $block)();
        serde_json::to_string(&result).unwrap()
    }};
}
pub(crate) use parsed_output;
