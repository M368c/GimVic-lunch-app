// If cookie exsist and is correct let the user in
auth_status()

async function auth_status() {
    const url = "http://127.0.0.1:3000/auth_status"; // HTTPS in production
    try {
        const response = await fetch(url, {
            method: "GET",
            credentials: "include",
        });
        if (response.ok) {
            // Go to main page
            window.location.href = window.location.href.replace("index.html", "pages/main.html");
        }
    }
    catch (err) {}
}


// POST request for users login
async function login() {
    const url = "http://127.0.0.1:3000/login"; // HTTPS in production
    try {
        const response = await fetch(url, {
            method: "POST",
            headers: {"Content-Type": "application/json"},
            credentials: "include",
            body: JSON.stringify({ username: document.getElementById("username").value, password: document.getElementById("password").value}),
        });
        if (response.ok) {
            const data = await response.json();
            localStorage.setItem('UserData', JSON.stringify(data.user));
            return true;
        }
        else {
            document.getElementById("errorLabel").textContent = 'Poskusi ponovno!';
            document.getElementById("errorIcon").innerHTML = '<i class="fas fa-exclamation-circle" style="color: red;"></i>';
            document.getElementById("username").value = "";
            document.getElementById("password").value = "";
            return false;
        }
    } 
    catch (error) {
        console.error(error);
        document.getElementById("errorLabel").innerHTML = 'Server is currently down!';
        return false;
    }
}

async function getLunchData() {
    const url = "http://127.0.0.1:3000/lunch_data";
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


// Submit btn pressed
window.addEventListener('load', function () { // Wait for page to load
    document.getElementById("submit_btn").addEventListener("click", submit);
})

async function submit() {
    // Send post request to backend
    let login_fn = await login();
    if (login_fn) {
        let data = await getLunchData();
        if (data) {
            // Go to main page
            window.location.href = window.location.href.replace("index.html", "pages/main.html");
            console.log(localStorage.getItem('savedLunchData'));
        }
        else {window.location.href = window.location.href.replace("pages/main.html", "index.html");}
    }    
}