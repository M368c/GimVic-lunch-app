import { server_url } from "./url.js";

// Check for correct auth cookie
export async function auth_status() {
    const currentPath = window.location.pathname;
    if (currentPath.includes("/pages/main.html")) {
        return;
    }

    const url = `${server_url}/api/auth_status`;
    try {
        const response = await fetch(url, {
            method: "GET",
            credentials: "include",
        });
        if (response.ok) window.location.href = "/pages/main.html";
    } catch (err) {}
}

// Login
export async function login() {
    const url = `${server_url}/api/login`;
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
        } else if (response.status == 502) {
            document.getElementById("errorLabel").textContent =
                "Strežnik se trenutno ne odziva!";
            return false;
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
        document.getElementById("errorLabel").textContent =
            "Strežnik se trenutno ne odziva!";
        return false;
    }
}

export async function change_password() {
    const url = `${server_url}/api/change_password`;
    try {
        const response = await fetch(url, {
            method: "POST",
            credentials: "include",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({
                password: document.getElementById("current_password_input")
                    .value,
                new_password:
                    document.getElementById("new_password_input").value,
            }),
        });
        if (response.ok) {
            document.getElementById("message_label").textContent =
                "Sprememba gesla uspešna. \n Preusmerjanje na glavno stran";
            return true;
        } else {
            document.getElementById("message_label").textContent =
                "Sprememba gesla ni uspela. \n Poskusite ponovno!";
            return false;
        }
    } catch (error) {
        document.getElementById("message_label").textContent =
            "Sprememba gesla ni uspela. \n Poskusite ponovno!";
        console.error(error);
        return false;
    }
}

export async function logout() {
    const url = `${server_url}/api/logout`;
    try {
        const response = await fetch(url, {
            method: "POST",
            credentials: "include",
            headers: {
                "Content-Type": "application/json",
            },
        });
        if (response.ok) {
            window.location.href = "/index.html";
            localStorage.clear();
        } else {
            if (confirm("Seja je potekla. Prijavite se znova!")) {
                window.location.href = "/index.html";
            }
        }
    } catch (error) {
        console.error(error);
    }
}
