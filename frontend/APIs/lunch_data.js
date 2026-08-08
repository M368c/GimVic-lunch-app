import { server_url, apiRequest, toast_notification } from "./server.js";

// Lunch data for frontend
export async function getLunchData() {
    const url = `${server_url}/api/lunch_data`;
    const { data, error } = await apiRequest("lunch data", url, {
        method: "GET",
        cache: "no-store",
        credentials: "include",
    });
    if (error == null) {
        localStorage.removeItem("savedLunchData");
        localStorage.setItem("savedLunchData", JSON.stringify(data));
    }
}

// Update database
export async function updateLunch(lunch_data) {
    const url = `${server_url}/api/update_lunch_data`;
    const { data, error } = await apiRequest("lunch update", url, {
        method: "POST",
        credentials: "include",
        headers: {
            "Content-Type": "application/json",
        },
        body: JSON.stringify(lunch_data),
    });
    if (error == null) {
        toast_notification("Spremembe shranjene", "Success");
    }
}
