mod bindings {
    wit_bindgen::generate!({
        world: "my-world",
        generate_all
    });

    struct Component {}
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

        fn append(&self, s:String) {
            self.data.borrow_mut().push(s);
        }
        
        fn to_string(&self) -> String {
            self.data.borrow().join("")
        }
    }

    impl Guest for Component{
        type ExampleList = ExampleList;
    }

    export!(Component);
}

mod bindings2 {
    wit_bindgen::generate!({
        world: "my-world2",
        generate_all
    });

    struct Component {}

    use std::collections::HashMap;
    use dummy::logger::logger::log;

    use chrono::{NaiveDate, Duration};
    use shift_calendar;

    fn calculate_weeks_delta_from_base(year: i32, month: u32, day: u32) -> Option<i64> {
        //     January 1970
        //          unix base
        //          v
        // Mo Tu We Th Fr Sa Su
        //           1  2  3  4 < base week = 0
        //  5  6  7  8  9 10 11               1
        // 12 13 14 15 16 17 18               2
        // 19 20 21 22 23 24 25               :
        // 26 27 28 29 30 31
        //
        // 1969/12/29 as week base
        let date1 = NaiveDate::from_ymd_opt(1969, 12, 29)
            .unwrap() /* safe unwrap */;

        if let Some(date2)  = NaiveDate::from_ymd_opt(year, month, day) {
            let diff: Duration = date2 - date1;
            let weeks = diff.num_weeks();

            Some(weeks)
        } else {
            None
        }

    }

    impl Guest for Component {
        fn to_upper(input:String) -> String {
            let mut a:HashMap<String, String> = HashMap::new();

            a.insert(input.clone(), input.to_uppercase().clone());
            log(&format!("{:?}", a));

            log(&format!("経過週数: {}", 
                calculate_weeks_delta_from_base(1970, 1, 5).unwrap()
            ));
            log(&format!("経過週数: {}", 
                calculate_weeks_delta_from_base(1970, 1, 11).unwrap()
            ));
            log(&format!("経過週数: {}", 
                calculate_weeks_delta_from_base(1970, 1, 12).unwrap()
            ));

            input.to_uppercase()
        }
    }

    export!(Component);
}
