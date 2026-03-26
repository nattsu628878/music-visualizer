# Rust移行プラン（Music Visualizer）

## 目的

現在は各ページで Svelte 側に処理ロジックが分散しているため、以下を目的に Rust 側へ段階的に移行する。

- 処理の一元化（解析・変換・保存）
- パフォーマンスと保守性の改善
- UI（表示）と処理（ドメインロジック）の責務分離

---

## 基本方針

### 1. 責務分離

- **Frontend (`src`)**
  - 役割: 画面表示、入力受付、状態表示
  - 原則: 重い処理ロジックを持たない
- **Backend (`src-tauri/src`)**
  - 役割: 解析、変換、保存/読込、レンダリング計算
  - `#[tauri::command]` をユースケース単位で提供

### 2. 「完全Rust化」ではなく段階移行

WebView描画を完全Rust化するのではなく、まずは処理ロジックを Rust に寄せる。

---

## 優先移行対象

1. ファイル入出力 / プロジェクト保存・読込
2. 音声解析（FFT、波形、特徴量）
3. 動画出力・変換（FFmpeg連携拡張）
4. レイヤー合成計算（座標、透明度、タイムライン）
5. MIDI解析（ノート/イベント列）

---

## フォルダ再編指針

## `src`（Frontend）

```text
src/
  routes/
    multi-view-composer/+page.svelte   # UIコンテナ中心
    guide/+page.svelte                 # 解説ページ
  lib/
    api/
      tauri/                           # invokeラッパ（直接invokeを散らさない）
    stores/                            # UI状態のみ
    models/                            # DTO型（Rust側と対応）
    components/                        # 純UIコンポーネント
    features/
      composer/
      files/
      export/
```

### `src` 側ルール

- `invoke()` の直書きを避け、`lib/api/tauri` 経由に統一
- UIは「入力→API呼び出し→結果表示」に限定
- ドメイン計算をSvelte内に持たない

---

## `src-tauri/src`（Backend）

```text
src-tauri/src/
  main.rs
  lib.rs
  commands/
    audio.rs
    midi.rs
    project.rs
    render.rs
    export.rs
  services/
    analyzer/
    encoder/
    ffmpeg/
    storage/
  domain/
    layer.rs
    project.rs
    timeline.rs
    settings.rs
  dto/
  errors/
```

### Rust側ルール

- コマンドは**ユースケース単位**
  - 例: `analyze_audio`, `build_frame_data`, `export_video`, `save_project`
- DTOを通してフロントと通信（`serde`）
- エラー型を共通化して返却

---

## API設計指針

- 小さすぎるコマンドに分割しすぎない
- 1コマンド = 1ユースケース（入力/出力を明確化）
- 命名はTS/Rustで揃える
  - 例: `ProjectState`, `LayerConfig`, `RenderFrameRequest`

---

## 段階的移行プラン

## Phase 1: 境界の整備

- `src/lib/api/tauri/*` を作成
- 既存Svelteの処理呼び出しをAPI経由に統一

## Phase 2: 保存・変換系のRust統一

- project save/load を Rust command 化
- 既存FFmpeg処理を export service に整理

## Phase 3: 音声解析のRust移行

- FFT/波形抽出を Rust 側へ
- フロントには解析結果DTOを返す

## Phase 4: レイヤー計算のRust移行

- レイアウト・透明度・時間軸計算を Rust 側に集約

## Phase 5: Svelteの表示専用化

- Svelte内の計算ロジック削除
- UI制御のみ保持

---

## 進め方の注意点

- 一気に移行しない（機能単位で切り替え）
- 各Phaseごとに動作検証を実施
- 既存UI仕様を崩さない（見た目は維持、内部実装を置換）
- 先に型定義（DTO）を固定してから移行すると手戻りが少ない

---

## 完了条件（Definition of Done）

- 主要処理が Rust command 経由で実行される
- フロントに重い処理ロジックが残っていない
- プロジェクト保存/読込・出力が安定動作する
- コード構成が上記ディレクトリ方針に沿っている

