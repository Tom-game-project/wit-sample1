use crate::async_example::async_mock::async_mock::async_mock::async_func;

wit_bindgen::generate!({
    world: "async-example-world",
    // 非同期にしたい関数の指定
    // 書式: import:<Namespace>:<Package>/<Interface>@<Version>#<Function>
    async: [
        "import:async-mock:async-mock/async-mock@0.1.0-alpha#async-func",
        // "export:async-example-func"
    ],
    generate_all
});

struct Component {}

use crate::async_example::dummy::logger::logger::log;

impl Guest for Component {
    fn async_example_func() -> String {
        let a = async_func();
        log("called async_example_func");
        return "String".to_string();
    }
}

export!(Component);
