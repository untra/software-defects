const TENANT_WEBHOOKS = {
    acme: { webhookUrl: "https://acme.example/hooks" },
    globex: { webhookUrl: "https://globex.example/hooks" },
};
class SettingsService {
    static cache = new Map();
    get(tenant, key) {
        const hit = SettingsService.cache.get(key);
        if (hit !== undefined)
            return hit;
        const value = TENANT_WEBHOOKS[tenant][key];
        SettingsService.cache.set(key, value);
        return value;
    }
}
const settings = new SettingsService();
console.log("acme   ->", settings.get("acme", "webhookUrl"));
console.log("globex ->", settings.get("globex", "webhookUrl"));
