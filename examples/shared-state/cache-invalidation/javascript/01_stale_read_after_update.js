const db = new Map([["ana", "free"]]);
const cache = new Map();
function getPlan(user) {
    const hit = cache.get(user);
    if (hit !== undefined)
        return hit;
    const plan = db.get(user) ?? "free";
    cache.set(user, plan);
    return plan;
}
function upgrade(user, plan) {
    db.set(user, plan);
}
getPlan("ana");
upgrade("ana", "pro");
console.log("ana's plan after paying:", getPlan("ana"));
