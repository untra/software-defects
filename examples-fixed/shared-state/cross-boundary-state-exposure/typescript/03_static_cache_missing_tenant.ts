const TENANT_WEBHOOKS: Record<string, Record<string, string>> = {
  acme: { webhookUrl: "https://acme.example/hooks" },
  globex: { webhookUrl: "https://globex.example/hooks" },
};

class SettingsService {
  static cache = new Map<string, string>();

  get(tenant: string, key: string): string {
    const cacheKey = JSON.stringify([tenant, key]);
    const hit = SettingsService.cache.get(cacheKey);
    if (hit !== undefined) return hit;
    const value = TENANT_WEBHOOKS[tenant][key];
    SettingsService.cache.set(cacheKey, value);
    return value;
  }
}

const settings = new SettingsService();
console.log("acme   ->", settings.get("acme", "webhookUrl"));
console.log("globex ->", settings.get("globex", "webhookUrl"));
