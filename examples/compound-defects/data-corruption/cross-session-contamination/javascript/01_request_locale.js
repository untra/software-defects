const DEFAULT_LOCALE = "en-US";
const context = { locale: DEFAULT_LOCALE };
function render(request) {
    if (request.locale !== undefined)
        context.locale = request.locale;
    return context.locale;
}
function main() {
    const first = { locale: "staff-preview" };
    render(first);
    console.log("next locale", render({}));
}
main();
