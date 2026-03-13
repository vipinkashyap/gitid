# GitID Desktop App — Design Handoff

> Developer specification document for the GitID Tauri desktop application.
> Covers design tokens, component specs, interaction behavior, states, edge cases, and accessibility.

---

## 1. Application Shell

### Window Configuration

| Property | Value |
|----------|-------|
| Default size | 900 x 700 px |
| Minimum size | 600 x 500 px |
| Resizable | Yes |
| Fullscreen | No |
| Tray icon | Yes (template icon on macOS) |

### Layout Structure

The app uses a single-window, fixed layout with three vertical zones:

```
┌──────────────────────────────────────────────────┐
│  HEADER  (border-b, px-6 py-4)                   │
│  [Logo 32x32] GitID / subtitle                   │
├──────────────────────────────────────────────────┤
│  TAB BAR  (px-6, border-b)                       │
│  Dashboard | Profiles | Rules | Doctor           │
├──────────────────────────────────────────────────┤
│  CONTENT  (px-6 py-6, max-w-4xl)                 │
│                                                  │
│  (tab content renders here)                      │
│                                                  │
│                                                  │
└──────────────────────────────────────────────────┘
```

The header is always visible. The tab bar and content area are replaced by the Setup Wizard on first run (when zero profiles exist).

### Header

| Element | Spec |
|---------|------|
| Logo container | `h-8 w-8 rounded-lg bg-primary` centered flex |
| Logo letter | "G", `text-primary-foreground font-bold text-sm` |
| Title | `text-lg font-semibold leading-none` — "GitID" |
| Subtitle | `text-xs text-muted-foreground` — "Multi-profile Git identity manager" |
| Gap between logo and text | `gap-3` (0.75rem) |

---

## 2. Design Tokens

### Color System

Uses **shadcn/ui HSL CSS variable** pattern. All colors are defined as HSL triplets (without `hsl()` wrapper) in CSS custom properties, then consumed via Tailwind as `hsl(var(--token))`.

#### Light Theme (`:root`)

| Token | HSL Value | Usage |
|-------|-----------|-------|
| `--background` | `0 0% 100%` | Page background |
| `--foreground` | `240 10% 3.9%` | Primary text |
| `--card` | `0 0% 100%` | Card background |
| `--card-foreground` | `240 10% 3.9%` | Card text |
| `--primary` | `240 5.9% 10%` | Primary buttons, active tab, logo bg |
| `--primary-foreground` | `0 0% 98%` | Text on primary |
| `--secondary` | `240 4.8% 95.9%` | Secondary buttons, badge bg |
| `--secondary-foreground` | `240 5.9% 10%` | Text on secondary |
| `--muted` | `240 4.8% 95.9%` | Muted backgrounds |
| `--muted-foreground` | `240 3.8% 46.1%` | Placeholder text, secondary text |
| `--accent` | `240 4.8% 95.9%` | Hover backgrounds |
| `--accent-foreground` | `240 5.9% 10%` | Text on accent |
| `--destructive` | `0 84.2% 60.2%` | Delete buttons, error text |
| `--destructive-foreground` | `0 0% 98%` | Text on destructive |
| `--border` | `240 5.9% 90%` | All borders |
| `--input` | `240 5.9% 90%` | Input borders |
| `--ring` | `240 5.9% 10%` | Focus ring color |
| `--radius` | `0.5rem` | Base border-radius |

#### Dark Theme (`.dark`)

| Token | HSL Value |
|-------|-----------|
| `--background` | `240 10% 3.9%` |
| `--foreground` | `0 0% 98%` |
| `--card` | `240 10% 3.9%` |
| `--primary` | `0 0% 98%` |
| `--primary-foreground` | `240 5.9% 10%` |
| `--secondary` | `240 3.7% 15.9%` |
| `--muted` | `240 3.7% 15.9%` |
| `--muted-foreground` | `240 5% 64.9%` |
| `--accent` | `240 3.7% 15.9%` |
| `--destructive` | `0 62.8% 30.6%` |
| `--border` | `240 3.7% 15.9%` |
| `--input` | `240 3.7% 15.9%` |
| `--ring` | `240 4.9% 83.9%` |

Dark mode is toggled via the `dark` class on the root element (`darkMode: "class"` in Tailwind config).

### Semantic Colors (Non-Token)

These use Tailwind's built-in palette directly (not CSS variables):

| Color | Class | Usage |
|-------|-------|-------|
| Emerald 400 | `text-emerald-400` | Success icons, "Active" text, all-clear banner |
| Emerald 500/15 | `bg-emerald-500/15 text-emerald-500` | Success badge |
| Amber 400 | `text-amber-400` | Warning icons, host rule color, sparkle icon |
| Amber 500/15 | `bg-amber-500/15 text-amber-500` | Warning badge |
| Red 400 | `text-red-400` | Error icons |
| Blue 400 | `text-blue-400` | Directory rule color, fix suggestions |
| Purple 400 | `text-purple-400` | Remote URL rule color |

### Typography

| Token | Value |
|-------|-------|
| Font family | `-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif` |
| Base text size | `text-sm` (0.875rem / 14px) — used in buttons, inputs, body text |
| Labels | `text-sm font-medium text-muted-foreground` |
| Helper text | `text-xs text-muted-foreground` (0.75rem / 12px) |
| Section headers | `text-xl font-semibold` |
| Card titles | `text-2xl font-semibold leading-none tracking-tight` |
| Inline card titles | `text-base` on CardTitle |
| Page title | `text-lg font-semibold leading-none` |
| Monospace | `font-mono text-xs` — for paths, patterns, SSH keys, emails |

### Spacing Scale

| Token | Value | Usage |
|-------|-------|-------|
| Page padding | `px-6 py-6` (1.5rem) | Content area |
| Header padding | `px-6 py-4` | Header |
| Card padding | `p-6` (header) / `p-6 pt-0` (content) | Standard cards |
| Compact card padding | `p-4` or `p-5` | Profile cards, check rows |
| Section gap | `space-y-6` (1.5rem) | Between major sections |
| Item gap | `space-y-3` or `space-y-2` | Between list items |
| Form field gap | `space-y-4` (1rem) | Between form fields |
| Grid gap | `gap-4` (1rem) / `gap-3` (0.75rem) | Stat cards, form grids |
| Button group gap | `gap-2` (0.5rem) | Between buttons |
| Icon gap | `gap-2` (0.5rem) | Between icon and text |

### Border Radius

| Token | CSS Value | Tailwind |
|-------|-----------|----------|
| `lg` | `var(--radius)` = 0.5rem | `rounded-lg` — Cards, dialogs |
| `md` | `calc(var(--radius) - 2px)` = 0.375rem | `rounded-md` — Buttons, inputs |
| `sm` | `calc(var(--radius) - 4px)` = 0.25rem | `rounded-sm` — Small buttons |
| `full` | 9999px | `rounded-full` — Badges, avatars |

### Scrollbar

Custom WebKit scrollbar:

| Property | Value |
|----------|-------|
| Width | 6px |
| Track | Transparent |
| Thumb | `hsl(var(--muted-foreground) / 0.3)` |
| Thumb hover | `hsl(var(--muted-foreground) / 0.5)` |
| Thumb radius | 3px |

---

## 3. Component Library

### Button

Five variants, four sizes. All have `transition-colors` and focus ring (`ring-2 ring-ring ring-offset-2`).

| Variant | Styles | Usage |
|---------|--------|-------|
| `default` | `bg-primary text-primary-foreground hover:bg-primary/90` | Primary actions (Create, Save, Import) |
| `secondary` | `bg-secondary text-secondary-foreground hover:bg-secondary/80` | Secondary actions |
| `destructive` | `bg-destructive text-destructive-foreground hover:bg-destructive/90` | Delete confirmation |
| `ghost` | `hover:bg-accent hover:text-accent-foreground` | Icon buttons, cancel, edit |
| `outline` | `border border-input bg-background hover:bg-accent` | Re-check, Install Helper, Retry |

| Size | Dimensions | Usage |
|------|------------|-------|
| `default` | `h-10 px-4 py-2` | Standard buttons |
| `sm` | `h-9 rounded-md px-3` | Compact buttons (Import, Edit) |
| `lg` | `h-11 rounded-md px-8` | Wizard CTA ("Continue to Dashboard") |
| `icon` | `h-10 w-10` | Icon-only actions (edit, delete, test SSH) |

**States:**
- **Disabled:** `pointer-events-none opacity-50`
- **Loading:** Spinner icon (`Loader2 animate-spin`) replaces or prepends the label
- **Focus:** `ring-2 ring-ring ring-offset-2 ring-offset-background`

### Input

Single variant. Height `h-10`, full width, `rounded-md`, `border border-input`.

**States:**
- **Placeholder:** `text-muted-foreground`
- **Focus:** `ring-2 ring-ring ring-offset-2`
- **Disabled:** `cursor-not-allowed opacity-50`
- **Compact variant:** Override with `h-8 text-sm` (used in wizard edit form)

### Card

`rounded-lg border bg-card text-card-foreground shadow-sm`

Composed of:
- `CardHeader`: `p-6`, flex column with `space-y-1.5`
- `CardTitle`: `text-2xl font-semibold leading-none tracking-tight`
- `CardContent`: `p-6 pt-0`

**Special card states:**
- **Empty state:** `p-8 text-center` with muted message and action button
- **All-clear (Doctor):** `border-emerald-500/20 bg-emerald-500/5` with emerald text
- **Imported (Wizard):** `border-emerald-500/30 bg-emerald-500/5` with check icon
- **Highlighted suggestion:** `border-primary/20`

### Badge

`rounded-full border px-2.5 py-0.5 text-xs font-semibold`

| Variant | Styles | Usage |
|---------|--------|-------|
| `default` | `bg-primary text-primary-foreground` | — |
| `secondary` | `bg-secondary text-secondary-foreground` | Profile name in rule rows |
| `destructive` | `bg-destructive text-destructive-foreground` | SSH test failed |
| `outline` | `border text-foreground` | Hosts, priority numbers, SSH key type |
| `success` | `bg-emerald-500/15 text-emerald-500 border-emerald-500/20` | "signed", "connected", active profile |
| `warning` | `bg-amber-500/15 text-amber-500 border-amber-500/20` | "no identity" |

### Select

Native `<select>` element. Same height and border styles as Input (`h-10 border border-input rounded-md`). Focus ring matches Input.

### Dialog (Modal)

| Property | Value |
|----------|-------|
| Overlay | `fixed inset-0 bg-black/80` |
| Container | `fixed inset-0 z-50 flex items-center justify-center` |
| Panel | `max-w-lg rounded-lg border bg-background p-6 shadow-lg` |
| Z-index | 50 |
| Close behavior | Click overlay calls `onClose` |

**Content patterns inside dialogs:**
- Profile form (create/edit)
- Delete confirmation (title + warning + Cancel/Delete buttons)
- Add Rule form
- Set Default Profile (select + Cancel/Set buttons)

### Tabs

Horizontal tab bar: `border-b border-border`, each tab is a `<button>`.

| State | Styles |
|-------|--------|
| Active | `border-b-2 border-primary text-foreground` |
| Inactive | `border-b-2 border-transparent text-muted-foreground hover:text-foreground` |

Tab items: `px-4 py-2.5 text-sm font-medium`, with `gap-2` between icon and label. Icons are 16x16 (`h-4 w-4`).

| Tab | Icon | View Component |
|-----|------|----------------|
| Dashboard | `LayoutDashboard` | `StatusBar` |
| Profiles | `Users` | `ProfileList` |
| Rules | `GitFork` | `RuleEditor` |
| Doctor | `Stethoscope` | `DoctorView` |

---

## 4. View Specifications

### 4.1 Loading State (App Boot)

Shown while checking if profiles exist. Centered on screen:

```
[pulsing "G" logo block — h-8 w-8 rounded-lg bg-primary animate-pulse]
```

### 4.2 Setup Wizard (First Run)

Shown when `getProfiles()` returns an empty map. Full-width centered layout (`max-w-2xl mx-auto`).

**Sections in order:**

1. **Hero** — centered icon (`h-14 w-14 rounded-2xl bg-primary/10`, Wand2 icon), title "Welcome to GitID" (`text-2xl font-bold`), subtitle varies based on detection results.

2. **Detection summary badges** — `flex flex-wrap gap-2 justify-center`. Each badge is `variant="outline"` showing: global name, global email, SSH key count, includeIf rule count, credential helper.

3. **SSH Keys card** — list of detected keys with path (monospace), key type badge, and comment.

4. **Suggested Profiles** — section header with Sparkles icon (amber). Each suggestion is a `SuggestionCard`:
   - **Collapsed:** Avatar circle (`h-9 w-9 rounded-full bg-primary/10`), name, badges for directory pattern, inline metadata (name, email, SSH key filename). Right side: Edit button (ghost, small) + Import button (default, small).
   - **Expanded:** 2-column grid below a `border-t`, with compact inputs (`h-8 text-sm`) for Profile ID, Git Name, Email, SSH Key (dropdown), Hosts, Directory Rule.
   - **Imported state:** Replaces the whole card with a success card (emerald border + bg, check icon, profile name, optional "+ directory rule" badge).

5. **No suggestions fallback** — centered card with message and no action.

6. **CTA** — centered `size="lg"` button: "Continue to Dashboard" or "Set Up Manually" + ChevronRight.

### 4.3 Dashboard (StatusBar)

Three sub-sections stacked vertically with `space-y-6`:

**OverviewStats** — `grid grid-cols-3 gap-4`. Each stat card:
- `p-4 text-center`
- Large number: `text-3xl font-bold`
- Label: `text-sm text-muted-foreground`
- Cards: Profile count, Active profile name (emerald), Match reason

**StatusCheck (Profile Resolution)** — Card with:
- Title: "Profile Resolution Check" with MapPin icon
- Input + Search button row (`flex gap-2`)
- Results panel: `rounded-lg border p-4 space-y-2`
  - Directory path in monospace
  - Profile badge (success) + reason text
  - Profile details shown with left border indicator (`border-l-2 border-border pl-2`)
  - Remote URL in monospace
  - If no profile: amber warning text

**RepoScanner** — Card with:
- Title: "Detect Repositories" with FolderSearch icon
- Input + Scan button
- Scrollable results area: `max-h-80 overflow-y-auto`
- Each repo row: `p-2.5 rounded-lg border`, showing repo name, remote URL (monospace), profile badge or email badge or "no identity" warning badge
- Footer: "Found N repo(s)" centered, `text-xs text-muted-foreground`
- Empty state: centered muted text

### 4.4 Profiles (ProfileList)

**Page header row:** title "Profiles" + subtitle on left, "New Profile" button (with Plus icon) on right.

**Empty state:** centered card with message and "Create Profile" button.

**Profile grid:** `grid gap-3`, each item is a `ProfileCard`.

**ProfileCard:**
- Card with `group` class for hover effects
- Content padding: `p-5`
- Left side: Profile ID as `text-lg font-semibold`, optional "signed" badge (success, Shield icon), then metadata rows with icons (User, Mail, Key, Globe) in `text-sm text-muted-foreground`
- SSH key paths in `font-mono text-xs`
- Host names as outline badges
- SSH test results: success/destructive badges per host
- **Hover actions (right side):** `opacity-0 group-hover:opacity-100 transition-opacity`
  - Test SSH (Wifi icon, ghost icon button) — only if SSH key + hosts exist
  - Edit (Pencil icon, ghost icon button)
  - Delete (Trash2 icon, ghost icon button, `text-destructive`)

**Profile Form (Dialog):**
- Title: "New Profile" or "Edit Profile"
- Fields: Profile ID (only on create), Git Name + Git Email (2-col grid), SSH Private Key Path, HTTPS Username + Associated Hosts (2-col grid), Signing Key + Signing Format (2-col grid)
- All labels: `text-sm font-medium text-muted-foreground`
- Error display: `text-sm text-destructive`
- Footer: Cancel (ghost) + Create/Save (default, with spinner when saving)

**Delete Confirmation (Dialog):**
- Title: "Delete Profile"
- Warning text with profile name in bold
- Cancel (ghost) + Delete (destructive)

### 4.5 Rules (RuleEditor)

**Page header row:** title + subtitle + "Add Rule" button.

**Priority explanation bar:** horizontal flex with `text-xs text-muted-foreground`, showing resolution order: Repo override → Directory (blue) → Remote URL (purple) → Host (amber) → Default. Each level has a numbered outline badge.

**Empty state:** centered card with "Add First Rule" button.

**Rule groups:** three sections (Directory, Remote, Host), each with a colored section header:

| Rule Type | Icon | Header Color |
|-----------|------|-------------|
| Directory | Folder | `text-blue-400` |
| Remote URL | Link | `text-purple-400` |
| Host | Globe | `text-amber-400` |

**RuleRow:**
- `p-3 rounded-lg border bg-card group hover:bg-accent/50 transition-colors`
- Grip handle (GripVertical, muted, `cursor-grab`) — visual only, no drag implemented
- Priority number in outline badge
- Rule type icon (colored)
- Pattern in `font-mono text-sm truncate` → arrow → profile badge (secondary)
- Sub-label: "{Type} rule" in `text-xs text-muted-foreground`
- **Hover actions:** `opacity-0 group-hover:opacity-100 transition-opacity`
  - Move up (ArrowUp, `h-8 w-8`, disabled at top)
  - Move down (ArrowDown, `h-8 w-8`, disabled at bottom)
  - Delete (Trash2, `h-8 w-8`, destructive color)

**Global Default card:**
- `p-4` compact card
- Shows current default profile as secondary badge, or "None set"
- "Change" outline button opens Set Default dialog

**Add Rule Form (Dialog):**
- Rule Type select (directory/remote/host, with priority hints in labels)
- Pattern input with dynamic placeholder based on type
- Helper text below pattern explaining match behavior
- Profile select dropdown
- Cancel + Add Rule buttons

### 4.6 Doctor (DoctorView)

**Page header row:** title "Doctor" with Stethoscope icon, subtitle, two outline buttons: "Install Helper" (Download icon, with loading spinner) and "Re-check" (RefreshCw icon, with loading spinner).

**Summary grid:** `grid grid-cols-3 gap-4`. Each card has icon + count + label:
- Passed: CheckCircle2 `h-8 w-8 text-emerald-400`, count `text-2xl font-bold`
- Warnings: AlertTriangle `h-8 w-8 text-amber-400`
- Errors: XCircle `h-8 w-8 text-red-400`

**Check results:** `space-y-2` list of `CheckRow`:
- `p-3 rounded-lg border`
- Status icon (emerald check / amber triangle / red X)
- Check name (`text-sm font-medium`) + status badge (success/warning/destructive)
- Message (`text-xs text-muted-foreground`)
- Fix suggestion if present: `text-xs text-blue-400 font-mono`

**All-clear banner:** appears only when zero errors and zero warnings. `border-emerald-500/20 bg-emerald-500/5`, centered emerald text: "All checks passed! GitID is healthy."

---

## 5. Interaction Specifications

### Transitions

| Element | Property | Duration | Easing |
|---------|----------|----------|--------|
| Button hover | `background-color` | Tailwind default (`150ms`) | `ease-in-out` |
| Tab switch | Content swap | Instant (no animation) | — |
| Hover action reveal | `opacity` | Tailwind default (`150ms`) | `ease-in-out` |
| Rule row hover | `background-color` | Tailwind default | `ease-in-out` |
| Chevron rotate (wizard edit) | `transform` | Tailwind default | — |
| Loading spinner | `rotation` | `animate-spin` (1s linear infinite) | Linear |
| Boot logo pulse | `opacity` | `animate-pulse` (2s cubic-bezier) | Cubic-bezier |
| Dialog open/close | None | Instant | — |

### Click / Tap Behavior

| Action | Behavior |
|--------|----------|
| Tab click | Immediately switch content; no loading indicator |
| "New Profile" / "Add Rule" | Open dialog with empty form |
| Profile card Edit | Open dialog with pre-filled form |
| Profile card Delete | Open delete confirmation dialog |
| Dialog overlay click | Close dialog (cancel action) |
| Cancel button in dialog | Close dialog |
| Form submit with validation error | Show error text below form, button returns to normal |
| "Import" on suggestion card | Replace card with success state (no dialog) |
| "Edit" expand on suggestion | Toggle edit fields with chevron rotation |
| Move Up / Move Down on rule | Reorder via API, then refresh list |
| Delete rule | Immediate delete (no confirmation dialog) |
| "Test SSH" on profile card | Show spinner on button, then show result badges |
| "Scan" on repo scanner | Populate scrollable list below |
| "Install Helper" | Show spinner, run API, refresh doctor checks |
| "Re-check" | Show spinner, re-run all doctor checks |

### Keyboard Interactions

| Key | Context | Behavior |
|-----|---------|----------|
| Enter | Form focused | Submit form |
| Escape | Dialog open | Close dialog |
| Tab | Any | Standard focus navigation |

---

## 6. Data Loading States

Every view that fetches data has three states:

### Loading

Centered spinner: `Loader2 h-6 w-6 animate-spin text-muted-foreground`, with `py-12` vertical padding.

Used in: ProfileList, RuleEditor, DoctorView.

### Error

Centered destructive text with error message and "Retry" outline button:
```
text-center py-12 text-destructive
"Failed to load {resource}: {error}"
[Retry button]
```

Used in: ProfileList, RuleEditor, DoctorView.

### Empty

Centered card (`p-8 text-center`) with muted descriptive text and a CTA button to create the first item.

Used in: ProfileList (no profiles), RuleEditor (no rules), RepoScanner (no repos found).

---

## 7. Edge Cases

### Content Limits

| Field | Max Length | Truncation |
|-------|-----------|------------|
| Profile ID | No explicit limit | Displayed as-is |
| Git Name | No explicit limit | Displayed as-is |
| Email | No explicit limit | Displayed as-is |
| SSH key path | No explicit limit | `font-mono text-xs`, overflows card |
| Remote URL | No explicit limit | `truncate` (single-line ellipsis) |
| Directory path | No explicit limit | `truncate` on most displays |
| Pattern in rule row | No explicit limit | `truncate` with `min-w-0` |
| Host badge list | No limit on count | Wraps horizontally with `gap-1.5` |
| Repo scanner results | No limit | `max-h-80 overflow-y-auto` |

### Missing Data

| Scenario | Behavior |
|----------|----------|
| Profile without SSH key | SSH row and Test SSH button not shown |
| Profile without hosts | Hosts row not shown |
| Profile without signing key | "signed" badge not shown |
| Profile without username | Field simply absent from display |
| Repo without remote URL | Remote URL line hidden |
| Repo with no profile or email | Shows "no identity" warning badge |
| Repo with email but no profile match | Shows email in outline badge |
| Status check with no profile resolved | Amber text: "No profile resolved for this directory." |
| Doctor check without fix suggestion | Fix line not shown |
| No detection results in wizard | Shows fallback card: "No existing Git identities detected" |
| Global identity missing name/email | Respective badges not shown in wizard summary |

### International Text

All text fields accept Unicode. No explicit i18n system — all UI strings are hardcoded English. Long international strings may exceed expected card widths; `truncate` class prevents overflow on most elements.

### Slow Connections / API Calls

All IPC calls go through Tauri `invoke()` to local Rust backend — latency is negligible (< 5ms). However, operations like SSH testing (`testSshConnection`) involve network calls and may take seconds. These show inline spinners on the triggering button.

`detectSetup()` involves filesystem scanning and may take 1-2 seconds on large machines. A dedicated full-screen loading state handles this.

---

## 8. Accessibility

### Current Implementation

| Feature | Status | Notes |
|---------|--------|-------|
| Semantic HTML | Partial | Uses `<button>`, `<form>`, `<input>`, `<select>`, `<label>`, `<h1>`-`<h3>` |
| ARIA labels | Missing | No `aria-label` on icon buttons (relies on `title` attribute) |
| Focus rings | Present | `ring-2 ring-ring ring-offset-2` on buttons and inputs |
| Focus order | Default | Tab order follows DOM order, which is logical |
| Color contrast | Adequate | Light theme meets AA; dark theme `muted-foreground` against dark bg may be borderline |
| Screen reader | Partial | Icon-only buttons have `title` but no `aria-label`; status icons lack `aria-label` |
| Keyboard nav | Partial | Forms work; dialogs don't trap focus; tabs not arrow-key navigable |
| Motion | Reduced | Only spinners and pulse; no `prefers-reduced-motion` handling |

### Recommended Improvements

1. **Add `aria-label` to all icon buttons.** Currently using `title` which is not announced by all screen readers. Map: Edit → "Edit profile {name}", Delete → "Delete profile {name}", Test SSH → "Test SSH connections for {name}", Move Up → "Increase priority", Move Down → "Decrease priority".

2. **Dialog focus trap.** When a dialog opens, focus should move to the first focusable element inside and be trapped until closed. On close, focus should return to the triggering element.

3. **Tab panel `role` attributes.** Add `role="tablist"` to the tab container, `role="tab"` to each tab button, `role="tabpanel"` to the content area. Use `aria-selected` and `aria-controls`/`id` linkage.

4. **Arrow key navigation for tabs.** Left/Right arrow keys should move between tabs when the tab bar is focused.

5. **Status icon descriptions.** Add `aria-label` to status icons in Doctor view: "Passed", "Warning", "Error".

6. **Announce dynamic content.** Use `aria-live="polite"` regions for: SSH test results, scan results, import confirmation, error messages.

7. **`prefers-reduced-motion` support.** Disable `animate-spin` and `animate-pulse` when the user has reduced motion enabled.

8. **Color independence.** Rule type colors (blue/purple/amber) are supplemented with icons and labels, which is good. Doctor status uses both color and icon. Ensure no information is conveyed by color alone.

---

## 9. Responsive Behavior

The app runs in a fixed desktop window (min 600x500). No mobile breakpoints needed, but content adapts to window resizing:

| Component | Behavior at min width (600px) |
|-----------|-------------------------------|
| Content area | `max-w-4xl` caps at large sizes; at 600px the full width is used |
| Stats grid | `grid-cols-3` stays; cards compress |
| Profile form | `grid-cols-2` may become tight; fields still usable |
| Rule priority bar | May overflow horizontally; no wrapping behavior |
| Rule row pattern | `truncate` activates earlier |
| Wizard suggestion cards | Metadata compresses; edit grid stays 2-col |
| Repo scanner list | `truncate` on repo names and URLs |

**Recommendation:** Add `@media (max-width: 640px)` overrides to switch form grids and stat grids to single-column at the minimum window size.

---

## 10. Icon Reference

All icons from **lucide-react**. Standard size is `h-4 w-4` (16x16). Large icons use `h-5 w-5` or `h-8 w-8`.

| Icon | Size | Usage |
|------|------|-------|
| LayoutDashboard | 4x4 | Dashboard tab |
| Users | 4x4 | Profiles tab |
| GitFork | 4x4 | Rules tab |
| Stethoscope | 4x4, 5x5 | Doctor tab, Doctor header |
| User | 3x3, 3.5x3.5, 4x4 | Name field in cards |
| Mail | 3x3, 3.5x3.5 | Email field |
| Key | 3x3, 3.5x3.5, 4x4 | SSH keys |
| Globe | 3x3, 3.5x3.5, 4x4 | Hosts, host rules |
| Shield | 3x3 | Signed badge |
| Wifi | 4x4 | Test SSH button |
| Plus | 4x4 | Add/Create buttons |
| Pencil | 4x4 | Edit button |
| Trash2 | 3.5x3.5, 4x4 | Delete button |
| Loader2 | 4x4, 6x6, 8x8 | Loading spinners (always with `animate-spin`) |
| Folder | 3x3, 4x4 | Directory rules, directory pattern badge |
| Link | 4x4 | Remote URL rules |
| GripVertical | 4x4 | Drag handle (visual only) |
| ArrowUp | 3.5x3.5 | Move rule up |
| ArrowDown | 3.5x3.5 | Move rule down |
| MapPin | 4x4 | Profile Resolution Check |
| FolderSearch | 4x4 | Repo scanner |
| GitBranch | 3x3, 4x4 | Repo rows, remote URL |
| Search | 4x4 | Search/check button |
| ExternalLink | — | Imported but unused |
| CheckCircle2 | 4x4, 8x8 | Doctor passed |
| AlertTriangle | 4x4, 8x8 | Doctor warning |
| XCircle | 4x4, 8x8 | Doctor error |
| RefreshCw | 4x4 | Re-check button |
| Download | 4x4 | Install Helper button |
| Wand2 | 7x7 | Wizard hero icon |
| Check | 4x4, 5x5 | Import button, imported confirmation |
| ChevronRight | 3x3, 4x4 | Wizard CTA, expand toggle |
| AlertCircle | 8x8 | Wizard error state |
| Sparkles | 4x4 | Suggested Profiles header |

---

## 11. IPC API Surface

The frontend communicates with the Rust backend via Tauri `invoke()`. All calls are typed in `tauri-api.ts`.

| Command | Parameters | Return Type | Used By |
|---------|-----------|-------------|---------|
| `get_profiles` | — | `Record<string, ProfileDto>` | ProfileList, RuleEditor, StatusBar, SetupWizard |
| `get_profile` | `name: string` | `ProfileDto` | — |
| `create_profile` | `name, profile` | `void` | ProfileList |
| `update_profile` | `name, profile` | `void` | ProfileList |
| `delete_profile` | `name` | `void` | ProfileList |
| `get_rules` | — | `RulesDto` | RuleEditor |
| `add_rule` | `ruleType, pattern, profile` | `void` | RuleEditor |
| `remove_rule` | `ruleType, index` | `void` | RuleEditor |
| `set_default_profile` | `profile` | `void` | RuleEditor |
| `reorder_rules` | `ruleType, newOrder` | `void` | RuleEditor |
| `get_status` | `path?: string` | `StatusDto` | StatusBar |
| `run_doctor` | — | `DoctorCheck[]` | DoctorView |
| `install_credential_helper` | — | `void` | DoctorView |
| `scan_repos` | `directory` | `DetectedRepo[]` | StatusBar |
| `test_ssh_connection` | `profileName` | `[string, boolean][]` | ProfileList |
| `detect_setup` | — | `DetectionResult` | SetupWizard |
| `import_suggested_profile` | `name, profile, directoryPattern` | `void` | SetupWizard |
| `guard_status` | — | `GuardStatusDto` | *Not yet wired to frontend* |
| `guard_install` | — | `void` | *Not yet wired* |
| `guard_uninstall` | — | `void` | *Not yet wired* |
| `get_suggestions` | `minEvidence` | `SuggestionDto[]` | *Not yet wired* |
| `get_activity_count` | — | `number` | *Not yet wired* |
| `apply_suggestion` | `ruleType, pattern, profile` | `void` | *Not yet wired* |

---

## 12. Pending Frontend Work

The following backend features are implemented but have no UI yet:

1. **Identity Guard panel** — needs a toggle switch for install/uninstall, status display, and a "check current repo" action. Suggested placement: new card in the Dashboard view or a dedicated sub-section.

2. **Pattern Learning / Suggestions** — needs a suggestions list with accept/dismiss actions, activity count badge, and possibly a "Learn" tab or Dashboard section.

3. **TypeScript bindings** — `tauri-api.ts` needs new types and functions for `GuardStatusDto`, `SuggestionDto`, and the 6 new IPC commands.

4. **Dark mode toggle** — the token system supports dark mode via `.dark` class, but no toggle UI exists. Consider a button in the header.
