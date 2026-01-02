import {
    $init,
    exampleResource,
    toUpper,
} from "./target/jco/component_features.js";

$init.then(() => {
    let example_resource = new exampleResource.ExampleList();

    example_resource.append("hello,");
    example_resource.append("world,");
    example_resource.append("tom,");

    let build_string = example_resource.toString();

    console.log(build_string);
    const button = document.getElementById('submit-btn');

    if (button instanceof HTMLButtonElement) {
        // このブロック内では button は HTMLButtonElement 型として扱われる
        button.addEventListener('click', (event: MouseEvent) => {
            // console.log('Clicked at', event.clientX, event.clientY);
            let upper = toUpper("hello world");
            console.log(upper);
            alert('Action performed via pure TypeScript and WebAssembly!');
        });
    } else {
        console.error('Button element not found or is not a button tag');
    }
})

