import { server_url } from "./url.js";

// Lunch data for frontend
export async function getLunchData() {
    const url = `${server_url}/api/lunch_data`;
    try {
        const response = await fetch(url, {
            method: "GET",
            credentials: "include",
        });
        if (response.ok) {
            const data = await response.json();
            localStorage.removeItem("savedLunchData");
            localStorage.setItem("savedLunchData", JSON.stringify(data));
            return true;
        } else {
            return false;
        }
    } catch (error) {
        return false;
    }
}

// Update database
export async function updateLunch(data) {
    const url = `${server_url}/api/update_lunch_data`;
    try {
        const response = await fetch(url, {
            method: "POST",
            credentials: "include",
            headers: {
                "Content-Type": "application/json",
            },
            body: data,
        });
        if (!response.ok) window.location.href = "/index.html";
    } catch (error) {
        document.getElementById("message_label").innerHTML =
            "Strežnik se trenutno ne odziva. Vpisani datumi ne bodo shranjeni!";
    }
}
