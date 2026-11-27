const ADMIN_TOKEN = "admin-secret";

type Context = { user: string; permissions: string[] };
type Request = { token?: string };

const ANONYMOUS_CONTEXT: Context = { user: "anonymous", permissions: [] };

function authenticate(req: Request, ctx: Context = ANONYMOUS_CONTEXT): Context {
  ctx = { ...ctx, permissions: [...ctx.permissions] };
  if (req.token === ADMIN_TOKEN) ctx.permissions.push("admin");
  return ctx;
}

authenticate({ token: ADMIN_TOKEN });
const anonymous = authenticate({});
console.log("anonymous request permissions:", anonymous.permissions);
