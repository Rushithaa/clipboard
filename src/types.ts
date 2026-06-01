export type ClipKind = "text" | "url" | "code" | "color" | "image";

export interface Clip {
  id: string;
  kind: ClipKind;
  /** Raw text, or a data:image/png;base64 URL for image clips. */
  content: string;
  preview: string;
  meta: Record<string, unknown>;
  sensitive: boolean;
  hash: string;
  created_at: string;
  workspace: string;
  tags: string[];
}

export type TransformOp =
  | "uppercase"
  | "lowercase"
  | "trim"
  | "url_encode"
  | "json_pretty";
