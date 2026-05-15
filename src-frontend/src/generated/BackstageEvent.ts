// Auto-generated from Rust BackstageEvent enum (src-tauri/src/window/mod.rs)

export type BackstageEvent =
  | { type: 'ContentChanged'; payload: { text: string; chapterId: string } }
  | { type: 'GenerationRequested'; payload: { chapterId: string; context: string } }
  | { type: 'FrontstageClosed' }
  | { type: 'FrontstageFocused' }
  | { type: 'DataRefresh'; payload: { entity: string } }
  | { type: 'NavigateTo'; payload: { view: string; highlightStoryId?: string; openPanel?: string } };
