class Widget {
  constructor() {
    this.render();
  }
  render() {}
}

class Label extends Widget {
  text = "default label";
  render() {
    console.log("rendering:", this.text);
  }
}

new Label();
