// Auto-generated from Rust FrontstageEvent enum (src-tauri/src/window/mod.rs)

export type FrontstageEvent =
  | { type: 'ContentUpdate'; payload: { text: string; chapterId: string } }
  | { type: 'AppendContent'; payload: { text: string; chapterId: string } }
  | { type: 'AiPreview'; payload: { text: string; insertPosition: number } }
  | { type: 'ChapterSwitch'; payload: { storyId: string; chapterId: string; title: string; content?: string } }
  | { type: 'SaveStatus'; payload: { saved: boolean; timestamp?: string } }
  | { type: 'DataRefresh'; payload: { entity: string } };
