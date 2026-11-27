const ADMIN_TOKEN = "admin-secret";
const ANONYMOUS_CONTEXT = { user: "anonymous", permissions: [] };
function authenticate(req, ctx = ANONYMOUS_CONTEXT) {
    if (req.token === ADMIN_TOKEN)
        ctx.permissions.push("admin");
    return ctx;
}
authenticate({ token: ADMIN_TOKEN });
const anonymous = authenticate({});
console.log("anonymous request permissions:", anonymous.permissions);
