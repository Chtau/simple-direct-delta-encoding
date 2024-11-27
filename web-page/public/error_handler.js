window.onerror = function(message, source, lineno, colno, error) {
    let errorElement = document.getElementById('error-display');
    if (errorElement) {
        errorElement.style.display = "flex";
        let errorTextElement = document.getElementById('error-display-text');
        errorTextElement.innerText = `Error: ${message} at ${source}:${lineno}:${colno}`;
    }
};