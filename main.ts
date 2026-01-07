import {
    $init,
    shiftManager,
} from "./target/jco/component_features.js";

$init.then(() => {
    let state = new shiftManager.ShiftManager();

    state.addNewGroup();
    // state.addNewGroup();

    //console.log(state.getStaffGroups());

    let submit_btn = document.getElementById("submit-btn");
    let add_slot_btn = document.getElementById("add-slot");

    submit_btn?.addEventListener("click", () => {
            console.log("submit_btn pushed");
            console.log(state.getStaffGroups());
    })

    add_slot_btn?.addEventListener("click", () => {
            console.log("add slot to 0")
            state.addSlot(0);
    })
})

