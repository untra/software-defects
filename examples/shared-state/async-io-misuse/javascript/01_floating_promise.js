const WRITE_LATENCY_MS = 10;
const profiles = new Map([["ana", "old bio"]]);
async function persistBio(user, bio) {
    await new Promise((resolve) => setTimeout(resolve, WRITE_LATENCY_MS));
    profiles.set(user, bio);
}
function updateBio(user, bio) {
    persistBio(user, bio);
    return "saved";
}
console.log(updateBio("ana", "new bio"));
console.log("profile page shows:", profiles.get("ana"));
