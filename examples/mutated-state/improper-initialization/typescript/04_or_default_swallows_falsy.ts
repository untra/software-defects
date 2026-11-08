const DEFAULT_RETRIES = 3;
const DEFAULT_LABEL = "untitled";

type JobOptions = { retries?: number; label?: string };

function configure(opts: JobOptions) {
  const retries = opts.retries || DEFAULT_RETRIES;
  const label = opts.label || DEFAULT_LABEL;
  return { retries, label };
}

console.log(configure({ retries: 0, label: "" }));
