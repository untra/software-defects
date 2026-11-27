class Formatter {
  format(s: string) { return s.trim(); }
  render(s: string) {
    return `[${this.format(s)}]`;
  }
}

class UpperFormatter extends Formatter {
  format(s: string) {
    return s.trim().toUpperCase();
  }
}

console.log(new UpperFormatter().render("  hello "));
