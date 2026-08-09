import { auth_status, login } from "./APIs/login.js";
import { getLunchData } from "./APIs/lunch_data.js";

// If cookie exsist and is correct let the user in
window.addEventListener("DOMContentLoaded", () => {
    auth_status();
});

const submit_btn = document.getElementById("submit_btn");
submit_btn.addEventListener("click", () => {
    submit();
});

async function submit() {
    // Send post request to backend
    let login_fn = await login();
    if (login_fn) {
        let data = await getLunchData();
        if (data) window.location.replace("/pages/main");
        else window.location.replace("/");
    }
}
