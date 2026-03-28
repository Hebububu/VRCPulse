# Design System — VRCPulse

## Product Context
- **What this is:** Real-time VRChat server status monitoring dashboard
- **Who it's for:** VRChat players who want ambient server health awareness, and Rust developers who appreciate well-built tools
- **Space/industry:** Server monitoring, status dashboards (peers: Grafana, Datadog, status.vrchat.com)
- **Project type:** Dashboard / web app (Tauri desktop + Axum web)

## Aesthetic Direction
- **Direction:** Industrial/Utilitarian
- **Decoration level:** Minimal
- **Mood:** A control room for VRChat. Function-first, data-dense, monospace accents. Think mission control, not marketing. The interface should feel fast, precise, and quietly confident.
- **Anti-patterns:** No gradients, no decorative shadows, no rounded corners, no colored-circle icons, no 3-column feature grids. Sharp corners everywhere.

## Typography
- **Display/Hero:** Geist Sans, 20px semibold — clean, modern, designed for developer tools
- **Body:** Geist Sans, 14px regular — readable at density, pairs naturally with Mono variant
- **UI/Labels:** Geist Sans, 12px medium — compact labels, navigation
- **Data/Tables:** Geist Mono, 24px bold (hero values), 14px regular (table cells) — terminal readout feel, tabular-nums built in
- **Axis/Meta:** Geist Mono, 11px regular — chart axes, timestamps, secondary data
- **Code:** Geist Mono
- **Loading:** Google Fonts CDN (`https://fonts.googleapis.com/css2?family=Geist+Sans:wght@400;500;600;700&family=Geist+Mono:wght@400;700`)
- **Scale:**
  - xs: 11px (axis labels, timestamps)
  - sm: 12px (UI labels, badges)
  - base: 14px (body, table cells)
  - lg: 16px (section headers)
  - xl: 20px (page title, status headline)
  - 2xl: 24px (hero metric values, Geist Mono)
  - 3xl: 32px (hero chart current value)

## Color
- **Approach:** Restrained — one accent + 4 semantic status colors on near-black
- **Background:** `#0f1117` — near-black, less harsh than pure black
- **Surface:** `#1a1d27` — card/panel background, one step up
- **Surface hover:** `#22252f` — interactive surface state
- **Border:** `#2a2d37` — thin structural dividers, 1px
- **Text primary:** `#e4e4e7` — light gray, high contrast on dark
- **Text secondary:** `#71717a` — muted, for labels and meta
- **Accent:** `#60a5fa` — muted blue, links, active states, chart primary line
- **Accent hover:** `#93bbfd` — lighter blue for hover states
- **Status operational:** `#22c55e` — green
- **Status minor:** `#eab308` — yellow
- **Status major:** `#f97316` — orange
- **Status critical:** `#ef4444` — red
- **Chart area fill:** accent at 15% opacity
- **Chart threshold:** status colors at 30% opacity for threshold zones
- **Dark mode:** This IS the primary theme. No light mode in v1.

## Spacing
- **Base unit:** 4px
- **Density:** Comfortable (not cramped, not spacious)
- **Scale:**
  - 2xs: 2px (micro gaps)
  - xs: 4px (tight padding, icon gaps)
  - sm: 8px (inner padding, between related items)
  - md: 16px (component padding, section gaps)
  - lg: 24px (between sections)
  - xl: 32px (major section breaks)
  - 2xl: 48px (page-level spacing)

## Layout
- **Approach:** Grid-disciplined
- **Grid:** 12-column, gap 16px
- **Max content width:** 1400px, centered
- **Min Tauri window:** 900x600px
- **Border radius:** 0px everywhere. Sharp corners only. This is the primary visual risk/differentiator.
- **Breakpoints:**
  - Desktop: 1200px+ (2-column chart grid)
  - Tablet: 768px-1199px (2-column, compressed)
  - Mobile: <768px (single column stack)
- **Status bar:** Full-width, 56px height, sticky top
- **Hero chart:** Full-width, 240px height (desktop), 180px (mobile)
- **Secondary charts:** 2-column grid, 160px height each
- **Incident feed:** Right sidebar on desktop (320px), below charts on mobile

## Motion
- **Approach:** Minimal-functional
- **Easing:** enter(ease-out) exit(ease-in) move(ease-in-out)
- **Duration:**
  - micro: 100ms (hover states, focus rings)
  - short: 200ms (color transitions, opacity)
  - medium: 300ms (chart data updates, layout shifts)
- **Chart update:** 300ms ease-out transition on new data points
- **Status change:** 200ms color fade on status bar
- **Data pulse:** Subtle opacity pulse (1.0 → 0.8 → 1.0, 400ms) when chart receives new data
- **No:** entrance animations, scroll effects, page transitions, loading spinners (use skeleton shimmer instead)

## Component Patterns
- **Cards:** No decorative cards. Surface-colored panels with 1px border. No shadow. No radius.
- **Buttons:** Ghost style (border only) for secondary actions. Accent fill for primary. No gradient.
- **Inputs:** Surface background, 1px border, no radius. Focus: accent border.
- **Badges:** Inline, small (12px font), status-colored background at 20% opacity with text in full color.
- **Tooltips:** Surface background, 1px border, anchored to crosshair on charts.
- **Skeleton loading:** Shimmer animation on surface-colored blocks. No spinners.
- **Segmented control:** For time range selector (1h/6h/12h/24h). Surface background, accent highlight on active segment. No radius.

## Chart Design
- **Line color:** Accent (#60a5fa) for primary metric
- **Area fill:** Accent at 15% opacity
- **Grid lines:** Border color (#2a2d37), 1px, dashed
- **Axis labels:** Geist Mono 11px, text-secondary color
- **Tooltip:** Surface panel with border, crosshair cursor, Geist Mono for values
- **Threshold zones:** Status color at 30% opacity (e.g., red zone above 5% error rate)
- **Y-axis formatting:** K suffix for thousands, ms suffix for latency, % for percentages
- **Time axis:** Adaptive density based on range (1h shows minutes, 24h shows hours)

## Decisions Log
| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-03-27 | Dark mode only for v1 | Matches VRChat aesthetic, monitoring dashboard convention |
| 2026-03-27 | Geist Sans + Geist Mono | Built for developer tools, tabular-nums support, not overused |
| 2026-03-27 | Zero border-radius | Industrial aesthetic differentiator, sharp corners = control room feel |
| 2026-03-27 | Geist Mono for data values | Terminal readout feel for hero metrics, distinctive vs typical dashboards |
| 2026-03-27 | Restrained color palette | Status colors need to be loud. Everything else stays quiet. |
| 2026-03-27 | No decorative elements | Minimal decoration. Typography and color do all the work. |
