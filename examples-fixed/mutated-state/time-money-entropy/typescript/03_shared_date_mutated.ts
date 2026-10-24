const ISO_DATE_LENGTH = 10;
const TRIAL_DAYS = 14;
const OCTOBER = 9;

function trialEnd(start: Date): Date {
  const end = new Date(start);
  end.setUTCDate(end.getUTCDate() + TRIAL_DAYS);
  return end;
}

const trialStart = new Date(Date.UTC(2026, OCTOBER, 1));
const end = trialEnd(trialStart);
console.log("start", trialStart.toISOString().slice(0, ISO_DATE_LENGTH));
console.log("end  ", end.toISOString().slice(0, ISO_DATE_LENGTH));
