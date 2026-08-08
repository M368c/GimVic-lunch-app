import { change_password } from "../APIs/login.js";
import { toast_notification } from "../APIs/server.js";

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
    window.location.replace("/pages/main.html");
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
            toast_notification("Gesli se ne ujemata!", "Warning");
        }
    } else toast_notification("Izpolni vsa polja!", "Warning");
}
