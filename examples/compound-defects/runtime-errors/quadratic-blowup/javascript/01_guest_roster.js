const GUESTS = ["Ari", "Bo", "Cy", "Dee"];
function roster(names) {
    const accepted = [];
    let comparisons = 0;
    for (const name of names) {
        let present = false;
        for (const earlier of accepted) {
            comparisons++;
            if (earlier === name)
                present = true;
        }
        if (!present)
            accepted.push(name);
    }
    return comparisons;
}
function main() {
    console.log("comparisons", roster(GUESTS));
}
main();
