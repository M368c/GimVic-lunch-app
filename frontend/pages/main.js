import { change_password, logout } from "../APIs/login.js";
import { getLunchData, updateLunch } from "../APIs/lunch_data.js";

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
        for (let i in JSON.parse(localStorage.getItem('calendarData'))) {
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
    if (selected) {
        selected.id = cancel_status;
        saveDateStatus(selected.textContent, 'cancel');
        selected.classList.remove('selected');
    }
});

getBtn.addEventListener('click', () => {
    let selected = datesElement.querySelector('.selected');
    if (selected) {
        selected.id = ok_status;
        saveDateStatus(selected.textContent, 'ok');
        selected.classList.remove('selected');
    }
});

// User profile
const userBtn = document.querySelector('.user-button');
const userDropdown = document.querySelector('.user-dropdown-menu');
let is_dropdown_open = false;

userBtn.addEventListener('click', () => {
    showUserProfile();
});

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

// Change password and login
const change_password_button = document.getElementById('change-password');
const logout_button = document.getElementById('logout');

change_password_button.addEventListener('click', () => {
    change_password();
});

logout_button.addEventListener('click', () => {
    logout();
});


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