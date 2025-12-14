import {
    $init,
    toUpper
} from "./dist/component_features.js";

$init.then(() => {
    let upper = toUpper("hello world");
    console.log(upper);
})

