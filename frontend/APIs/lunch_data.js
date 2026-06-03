import { server_url } from "./url.js";

// Lunch data for frontend
export async function getLunchData() {
    const url = `${server_url}/api/lunch_data`;
    try {
        const response = await fetch(url, {
            method: "GET",
            //cache: "no-store",
            credentials: "include",
            //headers: { "Cache-Control": "no-cache" },
        });
        if (response.ok) {
            const data = await response.json();
            localStorage.removeItem("savedLunchData");
            localStorage.setItem("savedLunchData", JSON.stringify(data));
        } else {
            document.getElementById("message_label").textContent =
                "Strežnik se trenutno ne odziva. Vpisani datumi ne bodo shranjeni!";
        }
    } catch (error) {
        document.getElementById("message_label").textContent =
            "Strežnik se trenutno ne odziva. Vpisani datumi ne bodo shranjeni!";
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
            body: JSON.stringify(data),
        });
        if (!response.ok) {
            document.getElementById("message_label").textContent =
                "Strežnik se trenutno ne odziva. Vpisani datumi ne bodo shranjeni!";
        }
    } catch (error) {
        document.getElementById("message_label").textContent =
            "Strežnik se trenutno ne odziva. Vpisani datumi ne bodo shranjeni!";
    }
}
