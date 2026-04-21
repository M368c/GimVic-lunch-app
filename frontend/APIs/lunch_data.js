import {server_url} from './url.js';

// Lunch data for frontend
export async function getLunchData() {
    const url = `${server_url}/lunch_data`;
    try {
        const response = await fetch(url, {
            method: "GET",
            credentials: "include",
        });
        console.log("Lunch data on login: ", response.status);
        if (response.ok) {
            const data = await response.json();
            localStorage.removeItem('savedLunchData');
            console.log("Lunch data are: ", JSON.stringify(data));
            localStorage.setItem('savedLunchData', JSON.stringify(data));
            return true;
        }
        else {return false;}
    }
    catch (error) {
        console.log("Couldn't get lunch data! (frontend)");
        console.error(error);
        return false;
    }
}

// Update database
export async function updateLunch(data) {
    const url = `${server_url}/update_lunch_data`;
    try {
        const response = await fetch(url, {
            method: "POST",
            credentials: "include",
            headers: {
                "Content-Type": "application/json"
            },
            body: JSON.stringify(data),
        });
        if (!response.ok) {
            window.location.href = window.location.href.replace("pages/main.html", "index.html");
        }
    } catch (error) {
        console.error(error);
        console.log("DEBUG: Check if backend is running!");
    }
}