const DATABASE_ADDRESS = "postgres://warehouse";
const PREFIX_LENGTH = 8;

function configure(): void {
  process.env.DB_URL = DATABASE_ADDRESS;
}

function addressPrefix(): string {
  return process.env.DATABASE_URL.slice(0, PREFIX_LENGTH);
}

function main(): void {
  const previous = { DB_URL: process.env.DB_URL, DATABASE_URL: process.env.DATABASE_URL };
  try {
    delete process.env.DATABASE_URL;
    configure();
    console.log("database", addressPrefix());
  } finally {
    for (const [key, value] of Object.entries(previous)) {
      if (value === undefined) delete process.env[key];
      else process.env[key] = value;
    }
  }
}

main();
