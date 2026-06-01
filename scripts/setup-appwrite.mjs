/**
 * One-time Appwrite provisioning for Smart Clipboard.
 *
 * Creates the database, `clips` collection, attributes and indexes.
 * The server API key is only used here — it is never bundled into the app.
 *
 * Usage (Node 20+):
 *   npm install            # installs node-appwrite (devDependency)
 *   cp .env.example .env    # fill in your values
 *   node --env-file=.env scripts/setup-appwrite.mjs
 */
import { Client, Databases, IndexType } from "node-appwrite";

const endpoint =
  process.env.APPWRITE_ENDPOINT || process.env.VITE_APPWRITE_ENDPOINT;
const project = process.env.APPWRITE_PROJECT || process.env.VITE_APPWRITE_PROJECT;
const apiKey = process.env.APPWRITE_API_KEY;
const dbId = process.env.VITE_APPWRITE_DB || "clipboard";
const collectionId = process.env.VITE_APPWRITE_COLLECTION || "clips";

if (!endpoint || !project || !apiKey) {
  console.error(
    "Missing config. Set APPWRITE_ENDPOINT/PROJECT and APPWRITE_API_KEY (see .env.example).",
  );
  process.exit(1);
}

const client = new Client()
  .setEndpoint(endpoint)
  .setProject(project)
  .setKey(apiKey);
const db = new Databases(client);

/** Run a step, ignoring "already exists" (409) errors so the script is idempotent. */
async function step(label, fn) {
  try {
    await fn();
    console.log(`✓ ${label}`);
  } catch (e) {
    if (e?.code === 409) {
      console.log(`• ${label} (already exists)`);
    } else {
      throw e;
    }
  }
}

async function main() {
  await step("database", () => db.create(dbId, "Clipboard"));
  await step("collection", () => db.createCollection(dbId, collectionId, "Clips"));

  await step("attr kind", () => db.createStringAttribute(dbId, collectionId, "kind", 16, true));
  await step("attr content", () => db.createStringAttribute(dbId, collectionId, "content", 1000000, false));
  await step("attr preview", () => db.createStringAttribute(dbId, collectionId, "preview", 512, false));
  await step("attr hash", () => db.createStringAttribute(dbId, collectionId, "hash", 128, true));
  await step("attr workspace", () => db.createStringAttribute(dbId, collectionId, "workspace", 64, false));
  await step("attr tags", () => db.createStringAttribute(dbId, collectionId, "tags", 64, false, undefined, true));
  await step("attr created_at", () => db.createStringAttribute(dbId, collectionId, "created_at", 64, false));

  // Indexes need the attributes to be available first.
  await new Promise((r) => setTimeout(r, 2000));
  await step("index hash", () => db.createIndex(dbId, collectionId, "by_hash", IndexType.Key, ["hash"]));
  await step("index workspace", () => db.createIndex(dbId, collectionId, "by_workspace", IndexType.Key, ["workspace"]));

  console.log("\nDone. Set VITE_APPWRITE_* in .env and run `npm run tauri dev`.");
}

main().catch((e) => {
  console.error("Setup failed:", e?.message || e);
  process.exit(1);
});
