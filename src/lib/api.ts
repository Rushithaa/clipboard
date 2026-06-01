import { invoke } from "@tauri-apps/api/core";
import type { Clip, TransformOp } from "../types";

export const api = {
  getClips: () => invoke<Clip[]>("get_clips"),
  deleteClip: (id: string) => invoke<void>("delete_clip", { id }),
  clearClips: () => invoke<void>("clear_clips"),
  getCaptureEnabled: () => invoke<boolean>("get_capture_enabled"),
  setCaptureEnabled: (enabled: boolean) =>
    invoke<void>("set_capture_enabled", { enabled }),
  getWorkspace: () => invoke<string>("get_workspace"),
  setWorkspace: (name: string) => invoke<void>("set_workspace", { name }),
  writeClipboard: (text: string) => invoke<void>("write_clipboard", { text }),
  transformText: (text: string, op: TransformOp) =>
    invoke<string>("transform_text", { text, op }),
};
