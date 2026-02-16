# Lumina

A modern presentation application for the GNOME desktop.

Lumina is inspired by Apple Keynote and provides a clean, intuitive interface
for creating presentations on Linux. Built with GTK4 and libadwaita, it
integrates seamlessly with the GNOME desktop environment.

## Features

- **Slide Management** -- Add, remove, duplicate, and reorder slides with
  drag-and-drop support
- **Text Elements** -- Rich text with configurable font family, size, bold,
  italic, color, and alignment
- **Shape Elements** -- Rectangles, ellipses, and lines with fill and stroke
  styling
- **Image Support** -- Insert PNG, JPEG, SVG, and WebP images
- **ODP Format** -- Save and load presentations in Open Document Presentation
  format, compatible with LibreOffice Impress
- **PPTX Import** -- Open PowerPoint files (read-only import)
- **PDF Export** -- Export presentations as multi-page PDF documents
- **Templates** -- Start new presentations from built-in templates (Blank,
  Title + Content, Photo Album)
- **Properties Panel** -- Edit position, size, font, colors, and stroke
  properties of selected elements
- **Internationalization** -- Available in English and German

## Screenshots

*Coming soon*

## Installation

### Flatpak (recommended)

*Flathub submission pending*

### Building from Source

#### Requirements

- Rust 1.80+
- Meson 0.63+
- GTK 4.12+
- libadwaita 1.5+
- Cairo, Pango, GDK-Pixbuf
- gettext

#### Build Steps

On Fedora / Fedora Toolbox:

```sh
sudo dnf install meson gcc gtk4-devel libadwaita-devel glib2-devel \
  pango-devel cairo-devel gdk-pixbuf2-devel gettext-devel rust cargo \
  desktop-file-utils appstream
```

Build and run:

```sh
meson setup build
meson compile -C build
./build/src/lumina
```

Install system-wide:

```sh
meson install -C build
```

## Keyboard Shortcuts

| Action          | Shortcut         |
|-----------------|------------------|
| New             | Ctrl+N           |
| Open            | Ctrl+O           |
| Save            | Ctrl+S           |
| Save As         | Ctrl+Shift+S     |
| Export as PDF   | Ctrl+Shift+E     |
| Quit            | Ctrl+Q           |
| Delete element  | Delete / Backspace |
| Deselect / Reset tool | Escape     |

## File Format Support

| Format | Read | Write |
|--------|------|-------|
| ODP    | Yes  | Yes   |
| PPTX   | Yes  | No    |
| PDF    | No   | Export |

## Technology

- **Language:** Rust
- **UI Toolkit:** GTK4 + libadwaita
- **Rendering:** Cairo + Pango
- **Build System:** Meson + Cargo
- **File Parsing:** quick-xml + zip

## License

Lumina is licensed under the [GNU General Public License v2.0](LICENSE).

## Author

Samuel Rueegger -- [rueegger.me](https://rueegger.me) -- samuel@rueegger.me
