import { Client, Account, Databases, ID, type Models } from "appwrite";
import type { Clip } from "../types";

/**
 * Appwrite sync layer (optional). Reads configuration from Vite env vars.
 * If the project is not configured, every function becomes a safe no-op so the
 * app remains fully usable offline / before credentials are provided.
 *
 * Configure via a `.env` file (see `.env.example`):
 *   VITE_APPWRITE_ENDPOINT, VITE_APPWRITE_PROJECT, VITE_APPWRITE_DB, VITE_APPWRITE_COLLECTION
 */
const endpoint = import.meta.env.VITE_APPWRITE_ENDPOINT as string | undefined;
const project = import.meta.env.VITE_APPWRITE_PROJECT as string | undefined;
const databaseId = import.meta.env.VITE_APPWRITE_DB as string | undefined;
const collectionId = import.meta.env.VITE_APPWRITE_COLLECTION as string | undefined;

export const appwriteEnabled = Boolean(
  endpoint && project && databaseId && collectionId,
);

let databases: Databases | null = null;
let account: Account | null = null;

if (appwriteEnabled) {
  const client = new Client().setEndpoint(endpoint!).setProject(project!);
  databases = new Databases(client);
  account = new Account(client);
}

/** Ensure there is an active Appwrite session (anonymous by default). */
export async function ensureSession(): Promise<Models.User<Models.Preferences> | null> {
  if (!appwriteEnabled || !account) return null;
  try {
    return await account.get();
  } catch {
    try {
      await account.createAnonymousSession();
      return await account.get();
    } catch (e) {
      console.warn("[appwrite] could not create session", e);
      return null;
    }
  }
}

/** Push a single clip to Appwrite. Sensitive clips are never synced. */
export async function pushClip(clip: Clip): Promise<void> {
  if (!appwriteEnabled || !databases || clip.sensitive) return;
  try {
    await databases.createDocument(databaseId!, collectionId!, ID.unique(), {
      kind: clip.kind,
      content: clip.kind === "image" ? "" : clip.content,
      preview: clip.preview,
      hash: clip.hash,
      workspace: clip.workspace,
      tags: clip.tags,
      created_at: clip.created_at,
    });
  } catch (e) {
    console.warn("[appwrite] push failed", e);
  }
}
