const server_url = "http://127.0.0.1";
export { server_url };

// Raw fetch request
export async function apiRequest(name, url, options) {
    try {
        const response = await fetch(url, options);

        if (!response.ok) {
            const error_msg = getErrorMessage(response.status);
            if (
                (name !== "login" ||
                    response.status === 429 ||
                    (response.status >= 500 && response.status < 600)) &&
                name !== "auth status"
            )
                toast_notification(error_msg, "Error");
            return {
                data: null,
                error: error_msg,
            };
        }

        // data handling
        let data = null;
        const text = await response.text();
        if (text) {
            try {
                data = JSON.parse(text);
            } catch {
                data = text;
            }
        }
        return { data, error: null };
    } catch (err) {
        console.error(err);
        if (name === "lunch data") {
            return { data: null, error: "Podatki nedosegljivi" };
        } else if (!navigator.onLine) {
            toast_notification(
                "Ni internetne povezave, spremembe ne bodo shranjene",
                "Error",
            );
            return { data: null, error: "Ni internetne povezave" };
        } else {
            toast_notification(
                "Strežnik ni dosegljiv, poskusi ponovno kasneje",
                "Error",
            );
            return {
                data: null,
                error: "Strežnik ni dosegljiv, poskusi ponovno kasneje",
            };
        }
    }
}

// Error messages for status codes
function getErrorMessage(status) {
    const messages = {
        400: "Nepravilni vnos podatkov.",
        401: "Potrebno se je prijaviti.",
        403: "Nimaš dovoljenja za to dejanje",
        404: "Stran ni najdena",
        429: "Preveč poslanih zahtev na strežnik. Poskusi kasneje.",
        502: "Strežnik trenutno ni dosegljiv.",
        503: "Strežnik trenutno ni dosegljiv.",
        504: "Strežnik se ne odziva pravočasno.",
    };
    return messages[status] || "Neznana težava, poskusi ponovno kasneje";
}

// Toast notifications
const isMobile = window.innerWidth < 800;
const notyf = new Notyf({
    types: [
        {
            type: "warning",
            duration: 5000,
            icon: false,
            backgroundColor: "orange",
        },
    ],

    duration: 3000,
    dismissible: false,
    icon: true,
    position: {
        x: "right",
        y: isMobile ? "bottom" : "top",
    },
});

export function toast_notification(message, type) {
    if (type === "Error") notyf.error(message);
    else if (type === "Success") notyf.success(message);
    else if (type === "Warning")
        notyf.open({
            type: "warning",
            message: message,
        });
}
