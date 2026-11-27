const settings = { debug: false };
function configure(options) {
  return options;
}
configure(settings).debug = true;
console.log(settings.debug);
