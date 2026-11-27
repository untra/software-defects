const BODY = '{"reading":"10"}';
const INITIAL_READING = 0;
const READING_PLACES = 1;
class Meter {
    reading = INITIAL_READING;
    load(body) {
        const record = JSON.parse(body);
        this.reading = record.reading;
    }
    display() {
        return this.reading.toFixed(READING_PLACES);
    }
}
function main() {
    const meter = new Meter();
    meter.load(BODY);
    console.log("reading", meter.display());
}
main();
