const DEFAULT_LOCALE = "en-US";
const context = { locale: DEFAULT_LOCALE };

type Request = { locale?: string };

function render(request: Request): string {
  if (request.locale !== undefined) context.locale = request.locale;
  return context.locale;
}

function main(): void {
  const first = { locale: "staff-preview" };
  render(first);
  console.log("next locale", render({}));
}

main();
