const kv = await Deno.openKv();
const analytics = [];

for await (const { key, value } of kv.list({ prefix: ["analytics"] })) {
    analytics.push({ key, value });
    await kv.delete(key);
}

const timestamp = new Date().toISOString();
const file = `export-${timestamp}.json`;
await Deno.writeTextFile(file, JSON.stringify(analytics));
