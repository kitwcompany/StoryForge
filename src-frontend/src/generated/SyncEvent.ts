// Auto-generated from Rust SyncEvent enum (src-tauri/src/state_sync/events.rs)
// Run `cargo test ts_export_tests -- --nocapture` in src-tauri to regenerate from ts-rs

export type SyncEvent =
  | { type: 'storyCreated'; payload: { storyId: string; title?: string } }
  | { type: 'storyUpdated'; payload: { storyId: string; title?: string } }
  | { type: 'storyDeleted'; payload: { storyId: string } }
  | { type: 'storySelected'; payload: { storyId: string; title?: string } }
  | { type: 'characterCreated'; payload: { storyId: string; characterId: string; name: string } }
  | { type: 'characterUpdated'; payload: { storyId: string; characterId: string; name?: string } }
  | { type: 'characterDeleted'; payload: { storyId: string; characterId: string } }
  | { type: 'sceneCreated'; payload: { storyId: string; sceneId: string; title?: string } }
  | { type: 'sceneUpdated'; payload: { storyId: string; sceneId: string; title?: string } }
  | { type: 'sceneDeleted'; payload: { storyId: string; sceneId: string } }
  | { type: 'sceneSelected'; payload: { storyId: string; sceneId: string; title?: string } }
  | { type: 'chapterCreated'; payload: { storyId: string; chapterId: string; title?: string } }
  | { type: 'chapterUpdated'; payload: { storyId: string; chapterId: string; title?: string } }
  | { type: 'chapterDeleted'; payload: { storyId: string; chapterId: string } }
  | { type: 'worldBuildingUpdated'; payload: { storyId: string } }
  | { type: 'characterRelationshipsUpdated'; payload: { storyId: string } }
  | { type: 'payoffLedgerUpdated'; payload: { storyId: string } }
  | { type: 'ingestionCompleted'; payload: { storyId: string; resourceType: string } }
  | { type: 'dataRefresh'; payload: { storyId?: string; resourceType: string } };
