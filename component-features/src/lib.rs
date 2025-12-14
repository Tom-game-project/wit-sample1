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
use std::collections::HashMap;

impl bindings::Guest for Component {
    fn to_upper(input:String,) -> String {
        let mut a:HashMap<String, String> = HashMap::new();

        a.insert(input.clone(), input.to_uppercase().clone());
        log(&format!("{:?}", a));
        input.to_uppercase()
    }
}

