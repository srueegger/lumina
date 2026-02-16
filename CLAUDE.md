# Lumina - Development Guide

## Project Overview
Lumina is a Keynote-inspired presentation application for the GNOME desktop, built with Rust + LibAdwaita. Default save format is ODP, with PPTX import and PDF export support.

## App Identity
- **App ID:** `me.rueegger.Lumina`
- **Binary:** `lumina`
- **License:** GPL-2.0-only

## Tech Stack
- Rust + gtk4-rs + libadwaita-rs
- Meson build system with Cargo integration
- Cairo for rendering (screen, thumbnails, PDF export)
- Pango for text layout
- quick-xml + zip for ODP/PPTX file format handling
- gettext for i18n (English + German)

## Development Environment
- Host: Bluefin Linux (immutable)
- Build: Fedora Distrobox (`lumina-dev`)
- Enter dev environment: `distrobox enter lumina-dev`

## Build Commands (inside distrobox)
```bash
meson setup build
meson compile -C build
./build/src/lumina
```

## Project Structure
- `src/` - Rust source code
  - `model/` - Document data model (slides, elements, styles)
  - `render/` - Cairo rendering engine
  - `format/` - File format handlers (ODP, PPTX, PDF)
  - `ui/` - GTK widgets and window layout
- `data/` - Desktop file, metainfo, icons, GResources
- `po/` - Translations (gettext)
- `build-aux/` - Build helper scripts

## Architecture
- Internal coordinates: Points (1/72 inch)
- Single `render_slide()` function shared by canvas, thumbnails, and PDF export
- Document model is pure Rust (serde-serializable), wrapped in `Rc<RefCell<>>` for UI

## Git Conventions
- Conventional commits: `feat:`, `fix:`, `chore:`, `docs:`, `refactor:`
- Language: English
- NO references to AI assistants or code generation tools in commits, code, or comments
- Co-Authored-By headers are NOT allowed

## Code Style
- Follow Rust idioms and clippy recommendations
- Use composite templates (.ui files) for GTK widget layouts
- Keep model/ independent of GTK (no GTK imports in model/)
- All user-facing strings must be translatable (gettext)
