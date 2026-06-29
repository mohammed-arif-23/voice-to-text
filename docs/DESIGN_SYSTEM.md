# ScribeRx Design System — "Calm Precision"

This document serves as the design specification and token source of truth for all UI components, stylesheets, and state views in ScribeRx (`ui/`).

---

## 1. Design Principles

1. **Invisible until needed**: Zero visual footprint when idle. Instant summon, instant dismissal.
2. **One focal action at a time**: Never clutter the popup with redundant controls.
3. **Motion communicates state**: Animations map to state changes (listening waveform, processing bar, smooth collapse).
4. **Trust through clarity**: Confidence is visually distinct and transparently editable.

---

## 2. Design Tokens

### 2.1 Color Palette

```css
:root {
  /* Surface & Blurs */
  --bg-surface-light: rgba(247, 248, 250, 0.85);
  --bg-surface-dark: rgba(11, 12, 14, 0.85);
  --backdrop-blur: 20px;

  /* Neutrals */
  --color-neutral-900: #0B0C0E;
  --color-neutral-700: #2D3139;
  --color-neutral-500: #6B7280;
  --color-neutral-300: #D1D5DB;
  --color-neutral-100: #F7F8FA;

  /* Clinical Accent */
  --color-accent-primary: #0A6CFF;
  --color-accent-hover: #0056D6;

  /* Confidence Colors */
  --color-confidence-high: transparent;
  --color-confidence-medium-underline: #C77B00;
  --color-confidence-low-bg: #FFF3DC;
  --color-confidence-low-border: #FFE0B2;
  --color-confidence-low-text: #8C4A00;

  /* Typography */
  --font-family-ui: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
  --font-family-mono: 'JetBrains Mono', monospace;
}
```

### 2.2 Geometry & Elevation

```css
:root {
  --radius-popup: 14px;
  --radius-chip: 8px;
  --shadow-popup: 0 8px 24px rgba(0, 0, 0, 0.18);
  --popup-width: 360px;
  --popup-height-default: 120px;
  --popup-height-expanded: 220px;
}
```

---

## 3. UI State Machines

| State | CSS Selector / Class | Visual Features | Interaction |
|---|---|---|---|
| **Idle** | `.state-idle` | `display: none` or opacity 0 | Unrendered / input pass-through |
| **Listening** | `.state-listening` | Active blue accent pulse, animated audio waveform bars | Press hotkey or pause to stop |
| **Processing** | `.state-processing` | Thin indeterminate progress line | Non-interactive brief delay |
| **Review (High Conf)** | `.state-review-high` | Clean JetBrains Mono transcript, flashes ~600ms | Auto-injects |
| **Review (Low Conf)** | `.state-review-low` | Popup expands to 220px, low-conf terms shown with yellow chips and 2-3 pill buttons | Arrow keys / Enter to pick alternative |
| **Injected** | `.state-injected` | 1px blue accent underline sweeps under popup, then fades down | Auto-dismiss |
