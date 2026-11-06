class Formatter {
    format = (s) => s.trim();
    render(s) {
        return `[${this.format(s)}]`;
    }
}
class UpperFormatter extends Formatter {
    format(s) {
        return s.trim().toUpperCase();
    }
}
console.log(new UpperFormatter().render("  hello "));
