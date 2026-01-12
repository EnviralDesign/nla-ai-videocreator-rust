# TESTING PROVIDER UI V2

## Status: Ready to Wire & Test

All V2 components compile successfully. Now need to wire into app.rs.

## What's Built:
✅ **ProvidersModalV2** - List + action buttons (Edit in Builder / Edit as JSON)  
✅ **ProviderJsonEditorModal** - Simple JSON editor  
✅ **ProviderBuilderModalV2** - Simplified builder (preserves UUID)

## Next Steps for Testing:

### 1. Wire V2 to app.rs

Find the Settings menu handler `on_open_providers` and switch it to use V2 modals:

```rust
// Add signals for V2
let mut show_providers_v2 = use_signal(|| false);
let mut show_json_editor = use_signal(|| false);
let mut show_builder_v2 = use_signal(|| false);
let mut edit_provider_path = use_signal(|| None::<PathBuf>);
let mut provider_files_v2 = use_signal(Vec::<PathBuf>::new);

// Handler to open providers modal
let open_providers_v2 = move |_| {
    provider_files_v2.set(list_global_provider_files());
    show_providers_v2.set(true);
};

// Render modals after main UI:
ProvidersModalV2 {
    show: show_providers_v2,
    provider_files: provider_files_v2,
    on_new: move |_| {
        edit_provider_path.set(None);
        show_builder_v2.set(true);
    },
    on_reload: move |_| {
        provider_files_v2.set(list_global_provider_files());
    },
    on_delete: move |path| {
        let _ = std::fs::remove_file(&path);
        provider_files_v2.set(list_global_provider_files());
    },
    on_edit_builder: move |path| {
        edit_provider_path.set(Some(path));
        show_builder_v2.set(true);
    },
    on_edit_json: move |path| {
        edit_provider_path.set(Some(path));
        show_json_editor.set(true);
    },
}

ProviderJsonEditorModal {
    show: show_json_editor,
    provider_path: edit_provider_path,
    on_saved: move |_| {
        show_json_editor.set(false);
        provider_files_v2.set(list_global_provider_files());
    },
}

ProviderBuilderModalV2 {
    show: show_builder_v2,
    provider_path: edit_provider_path,
    on_saved: move |_| {
        show_builder_v2.set(false);
        provider_files_v2.set(list_global_provider_files());
    },
}
```

### 2. Test Flow

1. Run `dx serve`
2. Click Settings → AI Providers
3. Create new provider:
   - Click "New"
   - Choose workflow
   - Name it "Test Provider"
   - Save
   - **Verify**: UUID is generated

4. Close & reopen app
5. Click Settings → AI Providers
6. Select "Test Provider"
7. Click "Edit in Builder"
   - **Verify**: Shows "Mode: Edit"
   - **Verify**: Workflow is loaded
   - **Verify**: Name is "Test Provider"
8. Save again
   - **Verify**: Same UUID (check JSON file)

9. Click "Edit as JSON"
   - **Verify**: JSON shows in editor
   - **Verify**: Can type without cursor jump
   - Edit name to "Test Provider 2"
   - Save
   
10. Reopen builder
    - **Verify**: Name is "Test Provider 2"
    - **Verify**: Still same UUID

## Expected Differences from V1:

- ✅ No cursor jump in JSON editor
- ✅ Builder opens in Edit mode correctly
- ✅ UUID preserved on every save
- ✅ Clean UX with equal status for Builder/JSON
- ✅ No draft buffer issues
- ✅ Direct file I/O, simple flow

## If It Works:

Delete old V1 files:
- `src/components/providers_modal.rs`
- `src/components/provider_builder_modal.rs`

Update imports in `components/mod.rs`.

Ship it!
