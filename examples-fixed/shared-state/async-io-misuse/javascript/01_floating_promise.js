const WRITE_LATENCY_MS = 10;
const profiles = new Map([["ana", "old bio"]]);
async function persistBio(user, bio) {
    await new Promise((resolve) => setTimeout(resolve, WRITE_LATENCY_MS));
    profiles.set(user, bio);
}
async function updateBio(user, bio) {
    await persistBio(user, bio);
    return "saved";
}
updateBio("ana", "new bio").then(result => {
    console.log(result);
    console.log("profile page shows:", profiles.get("ana"));
});
