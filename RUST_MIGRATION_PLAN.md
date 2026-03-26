# Rust Migration Plan (Phase 0 / Phase 1)

このドキュメントは、現行の `music-visualizer` を
- UI: Svelte
- 処理: Rust (Tauri command)
へ段階的に移行するための、**実装可能なタスク分解**です。

---

## Goal

- `src/routes/multi-view-composer/+page.svelte` に集中しているロジックを、Rust command 経由へ移す
- まずは **Phase 0 (契約定義)** と **Phase 1 (I/O + Project save/load)** を完成させる
- 動作を壊さず、段階移行で進める

---

## Current Baseline

- Frontend: SvelteKit + TypeScript
- Desktop: Tauri v2
- 既存 Rust command: `convert_video`, `check_ffmpeg_installed` (`src-tauri/src/lib.rs`)
- Composer 実装: `src/routes/multi-view-composer/+page.svelte`

---

## Target Structure

### Rust (`src-tauri/src`)

```text
src-tauri/src/
  lib.rs
  commands/
    mod.rs
    project.rs
    media.rs
  domain/
    mod.rs
    project.rs
    media.rs
    error.rs
  services/
    mod.rs
    project_store.rs
```

### Frontend (`src/lib/features/composer`)

```text
src/lib/features/composer/
  api/
    tauri.ts
  types/
    project.ts
    media.ts
  mappers/
    projectMapper.ts
```

---

## Phase 0: Contract Freeze (Schema + API)

### P0-1: Project schema definition

**Create**
- `src/lib/features/composer/types/project.ts`
- `src-tauri/src/domain/project.rs`

**Fields (minimum)**
- `projectVersion: number`
- `globalSettings`
  - `aspectRatio`, `resolution`, `backgroundColor`
  - `exportFormat`, `convertAfterRecording`, `frameRate`, `quality`
- `layers[]`
  - `id`, `type`, `name`, `visible`, `opacity`
  - `x`, `y`, `width`, `height`
  - `settings` (mode-specific object)
  - `assignedFileId?`
- `files[]` (metadata only for save/load contract)
  - `id`, `name`, `type`, `duration?`, `size`

**Acceptance**
- TS と Rust の型定義が 1:1 で対応
- 将来拡張のため `projectVersion` を必須化

---

### P0-2: Command I/O contract

**Create**
- `src/lib/features/composer/types/media.ts`
- `src-tauri/src/domain/media.rs`

**Define commands (Phase 1 scope)**
- `save_project(payload) -> saved_path`
- `load_project(path) -> project_payload`
- `pick_media_file() -> picked_path?`
- `read_media_metadata(path) -> metadata`

**Acceptance**
- 入出力 JSON の shape を固定
- エラー返却形式を固定（`code`, `message`, `details?`）

---

### P0-3: Error model

**Create**
- `src-tauri/src/domain/error.rs`

**Rules**
- command からは `Result<T, String>` でも可（初期）
- ただし内部では error kind を定義して map する

**Acceptance**
- UIで表示しやすいメッセージへ統一

---

## Phase 1: I/O + Project Save/Load

### P1-1: Rust module split

**Refactor**
- `src-tauri/src/lib.rs`
  - command 登録のみ残す
- `src-tauri/src/commands/mod.rs`
- `src-tauri/src/commands/project.rs`
- `src-tauri/src/commands/media.rs`
- `src-tauri/src/services/project_store.rs`

**Acceptance**
- 既存 command (`convert_video`, `check_ffmpeg_installed`) は動作維持
- 新 command を同時登録できる

---

### P1-2: Save project command

**Implement**
- command: `save_project`
- 入力: Project payload (JSON)
- 処理:
  1. save dialog path を受ける（もしくは frontend 側で path を先に取得）
  2. JSON serialize
  3. file write

**Frontend**
- `src/lib/features/composer/api/tauri.ts` に `saveProject()`
- `+page.svelte` から直接保存処理を外し、API呼び出しへ置換

**Acceptance**
- 現在の composer 状態が JSON で保存できる

---

### P1-3: Load project command

**Implement**
- command: `load_project`
- 入力: file path
- 処理:
  1. file read
  2. JSON parse
  3. schema validation (minimum keys check)
  4. payload return

**Frontend**
- `loadProject()` API 追加
- 受け取った payload を local state に復元

**Acceptance**
- 保存済み project をロードして layers/global settings が復元される

---

### P1-4: Media pick + metadata

**Implement**
- command: `pick_media_file`
- command: `read_media_metadata`
  - extension 判定 (`audio`/`midi`)
  - name/size/duration(可能なら)

**Frontend**
- `handleFileSelect()` を Rust command ベースに切り替え
- UIの `loadedFiles` は Rust結果を採用

**Acceptance**
- Filesパネル追加フローがJSローカル処理から Rust command 経由になる

---

### P1-5: Migration-safe adapter layer

**Create**
- `src/lib/features/composer/mappers/projectMapper.ts`

**Role**
- 旧 state shape と新 schema の吸収
- 不足フィールドの default 補完

**Acceptance**
- 旧データ/新データのどちらでもクラッシュせず表示できる

---

## Concrete Task List (ordered)

1. [ ] TS/Rust の `Project` 型作成 (`types/project.ts`, `domain/project.rs`)
2. [ ] TS/Rust の `MediaMetadata` 型作成 (`types/media.ts`, `domain/media.rs`)
3. [ ] Rust command modules 分割 (`commands/*`, `services/*`)
4. [ ] `save_project` 実装 + フロントAPI追加
5. [ ] `load_project` 実装 + フロント復元処理追加
6. [ ] `pick_media_file` / `read_media_metadata` 実装
7. [ ] `handleFileSelect` を Rust command 経由へ置換
8. [ ] manual regression test (Composer で追加/保存/読込/プレビュー)

---

## Test Checklist (Phase 1 done criteria)

- [ ] App起動後、Composerでレイヤー追加できる
- [ ] Save Project で JSON 保存できる
- [ ] Load Project で layers/global settings が復元される
- [ ] Files 追加が Rust command 経由で成功する
- [ ] 既存の `convert_video` と `check_ffmpeg_installed` が壊れていない

---

## Non-goals (Phase 0/1)

- FFT/描画ロジックの Rust 完全移行（Phase 2以降）
- MediaRecorder の撤廃（Phase 3以降）
- UI コンポーネント全面再設計

---

## Next (Phase 2 preview)

- 波形/スペクトラム計算を Rust へ
- フロントは描画専用（データ受け取りのみ）
- `multi-view-composer/+page.svelte` のロジック行数を半減させる

