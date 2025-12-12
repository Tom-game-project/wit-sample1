mod bindings {
    wit_bindgen::generate!({
        path: "./wit",
        generate_all
    });
    use super::Component;
    export!(Component);
}

struct Component;

use bindings::dummy::logger::logger::log;
impl bindings::Guest for Component {
    fn to_upper(input:String,) -> String {
        log(&input);
        input.to_uppercase()
    }
}

