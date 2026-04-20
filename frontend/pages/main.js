const monthYearElement = document.getElementById('monthYear');
const monthYearDateElement = document.getElementById('monthYear-yy-mm');
const datesElement = document.getElementById('dates');
const prevBtn = document.getElementById('prevBtn');
const nextBtn = document.getElementById('nextBtn');

// Buttons
const cancelBtn = document.getElementById('cancelBtn');
const getBtn = document.getElementById('getBtn');
const saveBtn = document.getElementById('saveBtn');

const cancel_status = `cancel`;
const ok_status = `ok`;

let currentDate = new Date();

const updateCalendar = () => {
    const currentYear = currentDate.getFullYear();
    const lastMonth = currentDate.getMonth();

    const firstDay = new Date(currentYear, lastMonth, 0); // first day
    const lastDay = new Date(currentYear, lastMonth + 1, 0); // +1 because they starts from 0
    const totalDays = lastDay.getDate();
    const firstDayIndex = firstDay.getDay(); // Day in the week
    const lastDayIndex = lastDay.getDay();

    const monthYearString = currentDate.toLocaleString
    ('default', {month: 'long', year: 'numeric'});
    const monthYearDateFormat = currentDate.toLocaleString
    ("sv-SE", {year: 'numeric', month: 'numeric'});

    monthYearElement.textContent = monthYearString;
    monthYearDateElement.textContent = monthYearDateFormat;

    let datesHTML = '';

    for (let i = 0; i < firstDayIndex; i++) {
        const prevDate = firstDay.getDate() - firstDayIndex + 1 + i;
        datesHTML += `<div class="date inactive">${prevDate}</div>`;
    }

    for(let i = 1; i<=totalDays; i++){
        const date = new Date(currentYear, lastMonth, i);
        const activeClass = date.toDateString() === new Date()
        .toDateString() ? 'active' : '';
        datesHTML += `<button id="${ok_status}" class="date ${activeClass}" >${i}</button>`;
    }

    for (let i = 1; i<=7-lastDayIndex; i++) {
        const nextDate = new Date(currentYear, lastMonth+1, i);
        datesHTML += `<div class="date inactive">${nextDate.getDate()}</div>`;
    }

    datesElement.innerHTML = datesHTML;

    restoreCalendarState();
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
        }
    }
    catch (error) {
        console.log("Couldn't get lunch data! (frontend)");
        console.error(error);
        return false;
    }
}

async function updateLunch(data) {
    const url = "http://127.0.0.1:3000/update_lunch_data"; // HTTPS in production
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
            window.location.href = window.location.href.replace("pages/main.html", "index.html"); // Back to login page
        }
    } catch (error) {
        console.error(error);
        console.log("DEBUG: Check if backend is running!"); // Remove for production
    }
}

async function change_password() {
    window.location.href = window.location.href.replace("pages/main.html", "pages/change_password.html");
    // Change password screen
    // Type old password and type new one
    // Update db with new password
    // Refresh the auth cookie
    const url = "http://127.0.0.1:3000/change_password";
    const response = await fetch(url, {
        method: "POST",
        credentials: "include",
        headers: {"Content-Type": "application/json"},
    });
}

async function logout() {
    const url = "http://127.0.0.1:3000/logout";
    const response = await fetch(url, {
        method: "POST",
        credentials: "include",
        headers: {
            "Content-Type": "application/json"
        },
    });
    if (response.ok) {
        window.location.href = window.location.href.replace("pages/main.html", "index.html");  // Go to login page
        localStorage.clear();
    }
    else {console.error("Couldn't logout!")}
}
prevBtn.addEventListener('click', () => {
    currentDate.setMonth(currentDate.getMonth()-1);
    updateCalendar();
})

nextBtn.addEventListener('click', () => {
    currentDate.setMonth(currentDate.getMonth()+1);
    updateCalendar();
})

// Selecting dates
datesElement.addEventListener('click', (event) => {
    if (event.target.classList.contains('date') && !event.target.classList.contains('inactive')) {
        const selected = datesElement.querySelector('.selected');
        if (selected) selected.classList.remove('selected');
        event.target.classList.add('selected');
    }
})

function restoreCalendarState() {
    if (localStorage.getItem('savedLunchData') != null) {
        const lunchData = JSON.parse(localStorage.getItem('savedLunchData'));
        const allDays = datesElement.querySelectorAll('.date:not(.inactive)');

        allDays.forEach(day => {
            // All numbers the same size
            let number = `${day.textContent}`;
            if (number / 10 < 1) {
                number = `0${day.textContent}`
            }

            for (const item of lunchData) {
                let value_string = item.date;
                if (monthYearDateElement.textContent == value_string.substring(0, 7) && number == value_string.substring(8, 10)){
                    day.id = cancel_status;
                }
            }
        });
    }
}

async function syncCalendarWithBackend() {
    const localData = localStorage.getItem('calendarData');
    const parsedData = JSON.parse(localData);
    await updateLunch(parsedData);
    await getLunchData();
    localStorage.setItem('calendarData', null);
}

// Convert in right format and store in local storage
function saveDateStatus(dateText, status) {
    // Format: yyyy-mm-dd
    var date_number = parseInt(dateText);
    if (date_number / 10 < 1) {
        dateText = `0${dateText}`
    }

    let calendarData = [];
    if (localStorage.getItem('calendarData')) {
        for (i in JSON.parse(localStorage.getItem('calendarData'))) {
            calendarData.push(i);
        }
    }

    const date = `${monthYearDateElement.textContent}-${dateText}`;

    if (status === 'ok'){
        calendarData.push(JSON.parse(`{"status":"ok", "date":"${date}"}`));
    }
    else if (status === 'cancel'){
        calendarData.push(JSON.parse(`{"status":"cancel", "date":"${date}"}`));
    }

    // Update the local storage
    localStorage.setItem('calendarData', JSON.stringify(calendarData));
    syncCalendarWithBackend();
}

// Main buttons for lunch handling
cancelBtn.addEventListener('click', () => {
    let selected = datesElement.querySelector('.selected');
    selected.id = cancel_status;
    saveDateStatus(selected.textContent, 'cancel');
    selected.classList.remove('selected');
});

getBtn.addEventListener('click', () => {
    let selected = datesElement.querySelector('.selected');
    selected.id = ok_status;
    saveDateStatus(selected.textContent, 'ok');
    selected.classList.remove('selected');
});

// User profile
const userBtn = document.querySelector('.user-button');
const userDropdown = document.querySelector('.user-dropdown-menu');
let is_dropdown_open = false;

function showUserProfile() {
    if (!is_dropdown_open) {
        userDropdown.style.display = "block";
        userBtn.style.border = "1px solid #e0e0e0";
        is_dropdown_open = true;
    }
    else {
        userDropdown.style.display = "none";
        userBtn.style.border = "none";
        is_dropdown_open = false;
    }
}

updateCalendar();

// Populate user profile with data
// Also need the class for all the users -- add to db
const userName = document.getElementById('user-name-label');
const userComputerName = document.getElementById('user-computer');
const userClass = document.getElementById('class-label');
const userUsername = document.getElementById('username-label');

const parsed_data = JSON.parse(localStorage.getItem('UserData'));
if (parsed_data == null) {
    logout();
}

const first_name = parsed_data.first_name;
const last_name = parsed_data.last_name;
const full_name = `${first_name} ${last_name}`;

userName.textContent = full_name;
userComputerName.textContent = full_name;
userUsername.textContent = JSON.parse(localStorage.getItem('UserData')).username;