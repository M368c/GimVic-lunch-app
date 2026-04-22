import { change_password } from "../APIs/login.js";

// User must populate all the inputs with data
// Verify that the password in 2nd and 3rd input are the same
// Check current password
// Change old password with new in db
// Return to pages/main.html

const submit = document.getElementById('submit_password_changes');
const cancel = document.getElementById('cancel_password_changes');

const current_password = document.getElementById('current_password_input');
const new_password = document.getElementById('new_password_input');
const again_new_password = document.getElementById('again_new_password_input');

submit.addEventListener('click', () => {
    if (current_password.value && new_password.value && again_new_password.value) {
        if (new_password.value === again_new_password.value) {
            await change_password(current_password, new_password);
        }
        else console.log("Passwords don't match!");
    }
    else console.log("You must fill out all the fields");
})