const DEFAULT_RETRIES = 3;
const DEFAULT_LABEL = "untitled";
function configure(opts) {
    const retries = opts.retries || DEFAULT_RETRIES;
    const label = opts.label || DEFAULT_LABEL;
    return { retries, label };
}
console.log(configure({ retries: 0, label: "" }));
