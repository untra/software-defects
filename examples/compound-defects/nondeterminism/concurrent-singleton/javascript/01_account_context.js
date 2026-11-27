class AccountContext {
    user = "";
    async show(user, ready) {
        this.user = user;
        await ready;
        return this.user;
    }
}
const context = new AccountContext();
function gate() {
    let open = () => undefined;
    const promise = new Promise((resolve) => { open = resolve; });
    return { promise, open };
}
async function main() {
    const firstReady = gate();
    const first = context.show("orchard", firstReady.promise);
    await context.show("harbor", Promise.resolve());
    firstReady.open();
    console.log("orchard received", await first);
}
void main();
