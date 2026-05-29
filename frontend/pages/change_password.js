import { change_password } from "../APIs/login.js";

// User must populate all the inputs with data
// Verify that the password in 2nd and 3rd input are the same
// Check current password
// Change old password with new in db
// Return to pages/main.html

const submit_btn = document.getElementById("submit_password_changes");
const cancel_btn = document.getElementById("cancel_password_changes");

const current_password = document.getElementById("current_password_input");
const new_password = document.getElementById("new_password_input");
const again_new_password = document.getElementById("again_new_password_input");

const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms));

submit_btn.addEventListener("click", () => {
    submit();
});

async function submit() {
    if (
        current_password.value &&
        new_password.value &&
        again_new_password.value
    ) {
        if (new_password.value === again_new_password.value) {
            let is_ok_changed = await change_password();
            if (is_ok_changed) {
                await sleep(1500);
                window.location.href = window.location.href.replace(
                    "pages/change_password.html",
                    "pages/main.html",
                );
            }
        } else {
            document.getElementById("message_label").innerHTML =
                "Passwords don't match!";
        }
    } else
        document.getElementById("message_label").innerHTML =
            "You must fill out all the fields";
}
