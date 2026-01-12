# Provider UI V2 - Architecture & Status

## Why V2?

The original provider modals had accumulated several critical issues:
1. **use_effect reactivity problems** - Effect wasn't re-running when seed changed
2. **Dual draft buffers** - JSON editor had local RefCell fighting with parent signals (causing cursor jump bug)
3. **State desync** - Multiple layers of signals getting out of sync
4. **UUID regeneration** - Builder creating new UUIDs instead of preserving existing ones
5. **UX**: Builder was hidden behind "Edit Build" button, should be first-class

## V2 Architecture Principles

### 1. **No Draft Buffers**
- Direct controlled inputs: `value={signal()}` `oninput={move |e| signal.set(e.value())}`
- No local RefCell caching for "stability"
- Signal is single source of truth

### 2. **No Effects for Initialization**
- Load data when modal opens using simple `use_effect` that just loads from file
- No complex "initialized" flags or state tracking
- Props/signals directly populate UI fields

### 3. **Explicit File I/O**
- Load: Read JSON file → parse → set signals
- Save: Read signals → serialize → write file
- No automatic syncing, no watchers

### 4. **Simplified Flow**
```
User clicks "AI Providers" in menu
    ↓
ProvidersModalV2 opens:
    - Left: Provider list (read from disk)
    - Right: When selected, show buttons:
        - "Edit in Builder" → opens ProviderBuilderModalV2
        - "Edit as JSON" → opens ProviderJsonEditorModal
    ↓
Edit in chosen modal
    ↓
Click Save → writes to disk
    ↓
Modal closes, list refreshes
```

## Files Created

### ✅ providers_modal_v2.rs
- Main modal with list + action buttons
- Simple selection state
- Event handlers for:  
  - `on_new` - Create new provider
  - `on_reload` - Refresh file list
  - `on_delete` - Delete selected provider
  - `on_edit_builder` - Open builder modal
  - `on_edit_json` - Open JSON editor

### ✅ provider_json_editor_modal.rs  
- Simple JSON editor
- Load from file on open
- Direct textarea editing
- Validate on save
- **No cursor jump bug** - single controlled input, no draft

### ⏳ provider_builder_modal_v2.rs (TODO)
- Load provider JSON from `provider_path` prop
- Parse and populate fields directly (no effects)
- Preserve UUID from existing provider
- Save writes provider + manifest to disk

## What's Left to Build

1. **ProviderBuilderModalV2** - Clean version of builder
   - Remove all `use_effect` initialization logic
   - Load/parse provider in component body when path changes
   - Simple controlled inputs for all fields
   - Preserve existing provider ID on save

2. **Wire V2 to app.rs** - Replace old modals with V2
   - Update Settings menu handler
   - Remove old modal signals/handlers
   - Add V2 modal signals/handlers

3. **Test & Verify**
   - Create provider → save → close app
   - Reopen → select → edit in builder → saves with same UUID
   - Edit as JSON → no cursor jump
   - Delete works

4. **Remove old modals** once V2 is verified working
   - Delete `providers_modal.rs`
   - Delete `provider_builder_modal.rs`
   - Update imports

## Key Differences from V1

| Aspect | V1 (Old) | V2 (New) |
|--------|----------|----------|
| **JSON Editor** | Inline with draft buffer | Separate modal, direct edit |
| **Builder Init** | Complex use_effect with initialized flag | Load in component body, no flag |
| **State Sync** | Multiple signal layers | File is source of truth |
| **UX** | JSON first-class, builder hidden | Both equal, user chooses |
| **Cursor Jump** | Yes (draft buffer issue) | No (single controlled input) |
| **UUID** | Sometimes regenerated | Always preserved |

## Migration Strategy

1. Build V2 alongside V1 (done for modal + JSON editor)
2. Create ProviderBuilderModalV2
3. Wire V2 to app
4. Test thoroughly
5. Once verified, delete V1 files
6. Ship it

## Status

- **ProvidersModalV2**: ✅ Complete
- **ProviderJsonEditorModal**: ✅ Complete  
- **ProviderBuilderModalV2**: ⏸️ Next task
- **Integration**: ⏸️ After builder complete
- **Testing**: ⏸️ After integration
- **V1 Removal**: ⏸️ After verification

*Last updated: 2026-01-09 16:59 CST*
