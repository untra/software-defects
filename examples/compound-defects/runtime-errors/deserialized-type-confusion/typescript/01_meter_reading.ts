const BODY = '{"reading":"10"}';
const INITIAL_READING = 0;
const READING_PLACES = 1;

class Meter {
  reading: any = INITIAL_READING;

  load(body: string): void {
    const record = JSON.parse(body);
    this.reading = record.reading;
  }

  display(): string {
    return this.reading.toFixed(READING_PLACES);
  }
}

function main(): void {
  const meter = new Meter();
  meter.load(BODY);
  console.log("reading", meter.display());
}

main();
