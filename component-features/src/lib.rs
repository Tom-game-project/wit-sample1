mod bindings {
    wit_bindgen::generate!({
        world: "my-world",
        // path: "./wit",
        generate_all
    });


    struct Component2 {}
    struct ExampleList {
        data: RefCell<Vec<String>>
    }

    use std::{
        cell::RefCell,
    };

    use exports::component::component_features::example_resource::{GuestExampleList, Guest};

    impl GuestExampleList for ExampleList {
        fn new() -> Self {
            Self {
                data: RefCell::new(vec![]) 
            }
        }

        fn append(&self, s:String,) {
            self.data.borrow_mut().push(s);
        }
        
        fn to_string(&self,) -> String {
            self.data.borrow().join("")
        }
    }

    impl Guest for Component2{
        type ExampleList = ExampleList;
    }

    export!(Component2);
}

mod bindings2 {
    wit_bindgen::generate!({
        world: "my-world2",
        generate_all
    });

    struct Component {

    }

    use std::collections::HashMap;
    use dummy::logger::logger::log;

    impl Guest for Component {
        fn to_upper(input:String,) -> String {
            let mut a:HashMap<String, String> = HashMap::new();

            a.insert(input.clone(), input.to_uppercase().clone());
            log(&format!("{:?}", a));
            input.to_uppercase()
        }
    }

    export!(Component);
}
