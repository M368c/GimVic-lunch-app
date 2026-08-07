import { change_password } from "../APIs/login.js";

const submit_btn = document.getElementById("submit_password_changes");
const cancel_btn = document.getElementById("cancel_password_changes");

const current_password = document.getElementById("current_password_input");
const new_password = document.getElementById("new_password_input");
const again_new_password = document.getElementById("again_new_password_input");

const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms));

submit_btn.addEventListener("click", () => {
    submit();
});

cancel_btn.addEventListener("click", () => {
    cancel();
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
                window.location.replace("/pages/main.html");
            }
        } else {
            document.getElementById("message_label").textContent =
                "Passwords don't match!";
        }
    } else
        document.getElementById("message_label").textContent =
            "You must fill out all the fields";
}

function cancel() {
    window.location.replace("/pages/main.html");
}
