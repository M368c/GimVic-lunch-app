import { server_url } from "./url.js";

// Check for correct auth cookie
export async function auth_status() {
    const url = `${server_url}/auth_status`;
    try {
        const response = await fetch(url, {
            method: "GET",
            credentials: "include",
        });
        if (response.ok) {
            // Go to main page
            window.location.href = window.location.href.replace(
                "index.html",
                "pages/main.html",
            );
        }
    } catch (err) {}
}

// Login
export async function login() {
    const url = `${server_url}/login`;
    try {
        const response = await fetch(url, {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            credentials: "include",
            body: JSON.stringify({
                username: document.getElementById("username").value,
                password: document.getElementById("password").value,
            }),
        });
        if (response.ok) {
            const data = await response.json();
            localStorage.setItem("UserData", JSON.stringify(data.user));
            return true;
        } else {
            document.getElementById("errorLabel").textContent =
                "Poskusi ponovno!";
            document.getElementById("errorIcon").innerHTML =
                '<i class="fas fa-exclamation-circle" style="color: red;"></i>';
            document.getElementById("username").value = "";
            document.getElementById("password").value = "";
            return false;
        }
    } catch (error) {
        console.error(error);
        document.getElementById("errorLabel").innerHTML =
            "Server is currently down!";
        return false;
    }
}

export async function change_password(current_password, new_password) {
    // Check if current_password is correct
    // Update db with new password
    // Refresh the auth cookie
    const url = `${server_url}/change_password`;
    try {
        const response = await fetch(url, {
            method: "POST",
            credentials: "include",
            headers: { "Content-Type": "application/json" },
        });
    } catch (error) {
        console.error(error);
    }
}

export async function logout() {
    const url = `${server_url}/logout`;
    try {
        const response = await fetch(url, {
            method: "POST",
            credentials: "include",
            headers: {
                "Content-Type": "application/json",
            },
        });
        if (response.ok) {
            window.location.href = window.location.href.replace(
                "pages/main.html",
                "index.html",
            ); // Go to login page
            localStorage.clear();
        } else {
            console.log("Couldn't logout!");
        }
    } catch (error) {
        console.error(error);
    }
}
