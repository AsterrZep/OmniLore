# OmniLore 📓🖋️

OmniLore is a premium, local-first, lightweight worldbuilding and writing environment designed to work efficiently in low-resource systems (such as Chromebooks running Crostini Linux). The application leverages **Tauri v2** and **Svelte 5** to provide rich visual dashboards, prose/screenplay editors, and interactive relational maps while maintaining a tiny memory footprint (<40MB RAM).

---

## 🚀 Key Features

*   **Prose & Screenplay Editor**: High-fidelity live rendering for Markdown (`.md`) and Fountain (`.fountain`) screenplay sheets. Features auto-completion for `@entities` and `#tags` with instant SQLite indexing.
*   **Smart Live Analysis**: Real-time extraction of entities, tag co-occurrence weights, scene statistics, and one-click quick registration forms.
*   **Dual-Engine 2D Relational Graph**: Interactive force-directed node diagram with standard SVG mode (zooming, panning, node physics) and software-rendered Canvas mode (disabling CPU-heavy effects for non-accelerated VMs/GPUs).
*   **Ramified Flowchart Canvas**: Branching narrative map supporting condition checks, connector labeling, coordinate saving, and custom sidebar inspection panels.
*   **Chronological Timeline**: Sorted list of historical/narrative events linked to specific database entities, complete with ascending/descending order and title/description search.

---

## 🛠️ Tech Stack & Harmony

1.  **Backend (Rust)**: High-speed local file reading/writing, relationship calculations, and SQLite connection management (`rusqlite`).
2.  **Bridge (Tauri v2)**: Replaces heavy Chromium shells (Electron) with native system WebViews, reducing packaging sizes to ~2.5MB (for Debian).
3.  **Frontend (Svelte 5 + TypeScript)**: Pure reactive component rendering without Virtual DOM overhead, keeping the interface fluid and responsive.
4.  **Database (SQLite)**: Local relational engine indexing entities, links, flow coordinates, and timeline records synced with flat files on disk.

---

## ⚡ Compilation & Performance Optimizations

To keep OmniLore's runtime lightweight and fast, the project includes specialized build profile optimizations in `Cargo.toml`:

```toml
[profile.release]
panic = "abort"      # Disables stack unwinding on panic, saving binary size
codegen-units = 1    # Enables maximum compiler optimizations across the crate
opt-level = "s"      # Optimizes for size (equivalent to -Os)
lto = true           # Enables Link-Time Optimization (LTO)
strip = true         # Strips all debug tables and symbols from the binary
```

This reduces the final compiled `.deb` package to **2.45MB** and restricts RAM consumption to **~35-40MB**, which is well within our `<150MB` target budget.

---

## 📦 Local Installation (Linux/ChromeOS)

To install the optimized Debian package locally on your ChromeOS Linux or Debian/Ubuntu machine:

1. Locate the built package in your home directory or copy it from `src-tauri/target/release/bundle/deb/`.
2. Run the following commands:
   ```bash
   sudo apt update
   sudo apt install ./OmniLore_0.1.0_amd64.deb
   ```
3. Run the application from your launcher or terminal using:
   ```bash
   OmniLore
   ```

---

## 🔄 CI/CD Release & Auto-Updater Configuration

The repository is configured with a GitHub Actions workflow under `.github/workflows/release.yml`. When you push a tag like `v0.1.0`, it automatically:
1. Compiles optimized binaries for Linux (`.deb` & `.AppImage`), Windows (`.msi`), and macOS (`.dmg`).
2. Drafts a release on your GitHub repository.
3. Automatically signs the bundles and generates the `latest.json` manifest required by the Tauri auto-updater.

### Setting Up Updater Keys

To enable signed updates, you must configure your personal updater keys:

1.  **Generate a signature keypair** on your local machine using the Tauri CLI:
    ```bash
    npx tauri signer generate
    ```
2.  **Configure the Public Key**:
    Copy the generated public key string and paste it in `src-tauri/tauri.conf.json` under:
    ```json
    "plugins": {
      "updater": {
        "pubkey": "YOUR_GENERATED_PUBLIC_KEY"
      }
    }
    ```
3.  **Configure GitHub Secrets**:
    Add the following repository secrets to your GitHub repository (`Settings -> Secrets and variables -> Actions`):
    *   `TAURI_SIGNING_PRIVATE_KEY`: Your generated private key text.
    *   `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`: (Optional) The password used to encrypt the private key.

Once configured, CI/CD runs will attach signature metadata directly to the draft releases, keeping your users secure and updated automatically.

---

## 💻 Local Development

To run a hot-reloading development environment:

```bash
# Install dependencies
npm install

# Start the Tauri development window
npm run tauri dev
```
