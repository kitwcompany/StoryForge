# Context: Frontend

Glossary for the React/TypeScript UI layer.

## Terms

| Term | Definition | Avoid using as synonym |
|------|------------|------------------------|
| **Frontstage** | Immersive writing interface. Warm paper tones, minimal chrome, zen mode. The author's "director chair". | "editor", "writing page" |
| **Backstage** | Professional studio management interface. Story/scene/character cards, knowledge graphs, settings. | "dashboard", "admin" |
| **Ghost Text** | Inline AI suggestion rendered as low-opacity text at cursor position. Tab to accept, Esc to reject. | "suggestion", "hint", "preview" |
| **WenSi (文思)** | AI creative assistance mode with three states: `off`, `passive`, `active`. Controls how proactively the AI offers help. | "AI mode", "assist mode" |
| **Slash Menu** | Inline `/` command palette in the editor. Commands: continue, polish, ancient-style, scene, auto-continue, proofread, comment, typeset. | "command palette", "menu" |
| **Data Refresh** | Standard event (`sync-event`) emitted by the backend; `useSyncStore` invalidates TanStack Query keys to sync state across windows. | "reload", "refresh" |
