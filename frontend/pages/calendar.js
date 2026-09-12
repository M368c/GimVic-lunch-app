const monthYearElement = document.getElementById("monthYear");
const monthYearDateElement = document.getElementById("monthYear-yy-mm");
const datesElement = document.getElementById("dates");
const prevBtn = document.getElementById("prevBtn");
const nextBtn = document.getElementById("nextBtn");

// Buttons
const cancelBtn = document.getElementById("cancelBtn");
const getBtn = document.getElementById("getBtn");
const saveBtn = document.getElementById("saveBtn");

const cancel_status = "cancel";
const ok_status = "ok";
const weekend_status = "weekend";
const holiday_status = "holiday";

const today = new Date();

let currentDate = new Date();

function updateCalendar() {
    const currentYear = currentDate.getFullYear();
    const lastMonth = currentDate.getMonth();

    const firstDay = new Date(currentYear, lastMonth, 0);
    const lastDay = new Date(currentYear, lastMonth + 1, 0);
    const totalDays = lastDay.getDate();
    const firstDayIndex = firstDay.getDay();
    const lastDayIndex = lastDay.getDay();

    const monthYearString = currentDate.toLocaleString("sl-SI", {
        month: "long",
        year: "numeric",
    });
    const monthYearDateFormat = currentDate.toLocaleString("sv-SE", {
        year: "numeric",
        month: "numeric",
    });

    monthYearElement.textContent = monthYearString;
    monthYearDateElement.textContent = monthYearDateFormat;

    let datesHTML = "";

    for (let i = 0; i < firstDayIndex; i++) {
        const prevDate = firstDay.getDate() - firstDayIndex + 1 + i;
        datesHTML += `<div class="date inactive">${prevDate}</div>`;
    }

    for (let i = 1; i <= totalDays; i++) {
        const date = new Date(currentYear, lastMonth, i);
        date.setHours(7, 55, 0, 0);
        const activeClass =
            date.toDateString() === new Date().toDateString() ? "active" : "";
        let weekend = "";
        let id = ok_status;

        if (date.getDay() === 6 || date.getDay() === 0) {
            weekend = "disabled";
            id = "weekend";
        }
        if (date < today || date - today <= 86400000) {
            datesHTML += `<button id="${id}" class="date ${activeClass} disabled" >${i}</button>`;
        } else {
            datesHTML += `<button id="${id}" class="date ${activeClass} ${weekend}" >${i}</button>`;
        }
    }

    for (let i = 1; i <= 7 - lastDayIndex; i++) {
        const nextDate = new Date(currentYear, lastMonth + 1, i);
        datesHTML += `<div class="date inactive">${nextDate.getDate()}</div>`;
    }

    datesElement.innerHTML = datesHTML;

    restoreCalendarState();
}

// User can view data just for current month and next one
prevBtn.addEventListener("click", () => {
    if (currentDate.getMonth() !== 8) {
        currentDate.setDate(1);
        currentDate.setMonth(currentDate.getMonth() - 1);
        updateCalendar();
    }
});
nextBtn.addEventListener("click", () => {
    if (currentDate.getMonth() + 1 <= new Date().getMonth() + 1) {
        currentDate.setDate(1);
        currentDate.setMonth(currentDate.getMonth() + 1);
        updateCalendar();
    }
});

// Selecting dates
datesElement.addEventListener("click", (event) => {
    if (
        event.target.classList.contains("date") &&
        !event.target.classList.contains("disabled") &&
        !event.target.classList.contains("inactive") &&
        !event.target.classList.contains("weekend")
    ) {
        const selected = datesElement.querySelector(".selected");
        if (selected) selected.classList.remove("selected");
        event.target.classList.add("selected");
    }
});

function restoreCalendarState() {
    if (localStorage.getItem("savedLunchData") != null) {
        const lunchData = JSON.parse(localStorage.getItem("savedLunchData"));
        const allDays = datesElement.querySelectorAll(".date:not(.inactive)");

        allDays.forEach((day) => {
            // All numbers the same size
            let number = `${day.textContent}`;
            if (number / 10 < 1) {
                number = `0${day.textContent}`;
            }

            let is_cancel = false;
            for (const item of lunchData) {
                let date = item.date;
                let status = item.status;
                if (
                    monthYearDateElement.textContent == date.substring(0, 7) &&
                    number == date.substring(8, 10)
                ) {
                    if (status == cancel_status) day.id = cancel_status;
                    if (status == holiday_status) day.id = holiday_status;
                    is_cancel = true;
                }
            }
            if (is_cancel === false && day.id !== weekend_status) {
                day.id = ok_status;
            }
        });
    }
}
