import { server_url, apiRequest, toast_notification } from "./server.js";

// Check for correct auth cookie
export async function auth_status() {
    const currentPath = window.location.pathname;
    if (currentPath.includes("/pages/main")) {
        return;
    }

    const url = `${server_url}/api/auth_status`;
    const { data, error } = await apiRequest("auth status", url, {
        method: "GET",
        credentials: "include",
    });
    if (error == null) {
        window.location.replace("/pages/main");
    }
}

// Login
export async function login() {
    const url = `${server_url}/api/login`;
    const { data, error } = await apiRequest("login", url, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        credentials: "include",
        body: JSON.stringify({
            username: document.getElementById("username").value,
            password: document.getElementById("password").value,
        }),
    });
    if (error == null) {
        localStorage.setItem("UserData", JSON.stringify(data.user));
        return true;
    } else {
        document.getElementById("errorLabel").textContent = "Poskusi ponovno!";
        document.getElementById("errorIcon").innerHTML =
            '<i class="fas fa-exclamation-circle" style="color: red;"></i>';
        document.getElementById("username").value = "";
        document.getElementById("password").value = "";
        return false;
    }
}

export async function change_password() {
    const url = `${server_url}/api/change_password`;
    const { data, error } = await apiRequest("change password", url, {
        method: "POST",
        credentials: "include",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
            password: document.getElementById("current_password_input").value,
            new_password: document.getElementById("new_password_input").value,
        }),
    });
    if (error == null) {
        toast_notification(
            "Sprememba gesla uspešna. Preusmerjanje na glavno stran",
            "Success",
        );
        return true;
    } else {
        return false;
    }
}

export async function logout() {
    const url = `${server_url}/api/logout`;
    const { data, error } = await apiRequest("logout", url, {
        method: "POST",
        credentials: "include",
        headers: {
            "Content-Type": "application/json",
        },
    });

    localStorage.clear();
    window.location.replace("/");
}
