# Wireframes (low fidelity)

Every screen has empty, loading, success, partial, and error states. The layouts below are
the default 1180×760 window; the 900×620 window collapses the sidebar labels and stacks
footer actions.

## Home / Import

```text
┌──────────────┬─────────────────────────────────────────────────────────────┐
│ Figma 2      │  Bring your Figma work to Penpot.              ● stays local │
│ Penpot       │  Drop your Figma files here. We’ll convert them and tell     │
│              │  you exactly what to check.                                  │
│ ① Import     │                                                             │
│ ② Files      │  ┌───────────────────────────────────────────────────────┐  │
│ ③ Details    │  │                                                       │  │
│ ④ Convert    │  │              ⬆  Drop your Figma files here            │  │
│ ⑤ Results    │  │                                                       │  │
│ ⑥ Report     │  │        [ Choose files… ]   [ Choose a folder… ]       │  │
│              │  │        Everything stays on your computer.             │  │
│ ──────────   │  └───────────────────────────────────────────────────────┘  │
│ Settings     │                                                             │
│ Preferences  │  1 Add files       2 Convert       3 Review in Penpot      │
│              │                                                             │
│              │  Last saved to  ~/Desktop/Penpot imports        Open folder │
└──────────────┴─────────────────────────────────────────────────────────────┘
```

Empty: drop zone only. Dragging: drop zone border + “Release to add these files”.
Error: banner “That file isn’t a .fig file. Choose a .fig file or a folder of them.”

## Files (queue)

```text
┌──────────────┬─────────────────────────────────────────────────────────────┐
│ nav          │  Your files                                  + Add files    │
│              │  Nothing is converted until you press Convert.              │
│              │  ┌───────────────────────────────────────────────────────┐  │
│              │  │ [icon] Landing page.fig       2 pages    Ready     ⋯ │  │
│              │  │        1.2 MB                                         │  │
│              │  ├───────────────────────────────────────────────────────┤  │
│              │  │ [icon] Design system.fig      14 pages   Ready     ⋯ │  │
│              │  └───────────────────────────────────────────────────────┘  │
│              │                                                             │
│              │  ┌───────────────────────────────────────────────────────┐  │
│              │  │ 2 files            Save to ~/Desktop/Penpot  Change  │  │
│              │  │                                         [ Convert ]  │  │
│              │  └───────────────────────────────────────────────────────┘  │
└──────────────┴─────────────────────────────────────────────────────────────┘
```

Empty: large neutral card with file icon, “No files yet.” + choose buttons.
Invalid row: warning icon + “Can’t be opened — try saving it again from Figma” + Remove.

## File details

```text
┌──────────────┬─────────────────────────────────────────────────────────────┐
│ nav          │  ← Files        What’s in this file?                       │
│              │  Landing page.fig · 1.2 MB                                  │
│              │  ┌─────────┬─────────┬─────────┬─────────┬─────────┐       │
│              │  │ Pages 2 │ Comps 8 │ Styles 5│ Vars 12 │ Images 4│       │
│              │  └─────────┴─────────┴─────────┴─────────┴─────────┘       │
│              │  Things to be aware of                                     │
│              │  ⚠ 2 Auto layout groups may resize differently              │
│              │  ⚠ 1 mixed text style will be flattened                     │
│              │  ▸ Technical details                                        │
│              │                                     [ Convert this file ] │
└──────────────┴─────────────────────────────────────────────────────────────┘
```

Unknown values show “Not available”. No guessed counts.

## Settings

```text
┌──────────────┬─────────────────────────────────────────────────────────────┐
│ nav          │  Conversion settings                                        │
│              │  Where should we save converted files?                      │
│              │  ~/Desktop/Penpot imports                         [ Change ]│
│              │                                                             │
│              │  What matters more?                                         │
│              │  ┌───────────────────────┐  ┌───────────────────────────┐ │
│              │  │ ● Looks identical     │  │ ○ Keeps tokens connected  │ │
│              │  │ Keep the visual result│  │ Prefer token/style links  │ │
│              │  └───────────────────────┘  └───────────────────────────┘ │
│              │                                                             │
│              │  ▸ Advanced                                                 │
│              │                                                  [ Done ]   │
└──────────────┴─────────────────────────────────────────────────────────────┘
```

## Converting

```text
┌──────────────┬─────────────────────────────────────────────────────────────┐
│ nav          │  Converting your files                        [Pause] [Cancel]
│              │  2 of 3 files finished                                      │
│              │  ┌───────────────────────────────────────────────────────┐  │
│              │  │ ✓ Reading your file            Landing page.fig  Done  │  │
│              │  │ ✓ Converting layers            Design system.fig  Done  │  │
│              │  │ ◉ Packaging for Penpot         Checkout.fig             │  │
│              │  │ ○ Checking the result                                  │  │
│              │  └───────────────────────────────────────────────────────┘  │
│              │  You can keep working. Cancelled files leave no output.     │
└──────────────┴─────────────────────────────────────────────────────────────┘
```

No percentage unless the true denominator is known. Pause stops new files; running files
finish. Cancel asks for confirmation.

## Results

```text
┌──────────────┬─────────────────────────────────────────────────────────────┐
│ nav          │  Your conversion is ready                                   │
│              │  2 converted · 1 needs a quick review · 0 couldn’t convert  │
│              │  ┌───────────────────────────────────────────────────────┐  │
│              │  │ ✓ Converted          Landing page.penpot   Open folder │  │
│              │  ├───────────────────────────────────────────────────────┤  │
│              │  │ ⚠ Needs a quick review  Checkout.penpot   3 to check   │  │
│              │  │   View report                                          │  │
│              │  └───────────────────────────────────────────────────────┘  │
│              │  In Penpot: Drafts → Options → Import Penpot files.         │
│              │  [ Open folder ]   [ View report ]   [ Retry failed ]       │
└──────────────┴─────────────────────────────────────────────────────────────┘
```

## Report

```text
┌──────────────┬─────────────────────────────────────────────────────────────┐
│ nav          │  Conversion report                         [Export ▾]       │
│              │  Checkout.fig · Converted — needs a quick review            │
│              │  42 layers converted · 3 need a quick review · 1 not         │
│              │                                                             │
│              │  [ Search… ]  [All severities ▾] [All categories ▾] [Page ▾]│
│              │  ▾ Needs your attention (1)                                 │
│              │    ⚠ Auto layout sizing changed        Frame / Card / Hero   │
│              │      What happened: …                                      │
│              │      What we did: …                                        │
│              │      In Penpot: …                              [Details ▾]  │
│              │  ▸ Slightly different (3)                                  │
│              │  ▸ Not converted (1)                                       │
│              │  ▸ All good (37)                                           │
└──────────────┴─────────────────────────────────────────────────────────────┘
```

Default: grouped by what to do. Technical code/node ID only under **Details**.
Empty: “Nothing needs your attention.” Validation not run: explicit notice.

## Preferences

```text
┌──────────────┬─────────────────────────────────────────────────────────────┐
│ nav          │  Preferences                                                │
│              │  Appearance      ( Follow system ) ( Light ) ( Dark )      │
│              │  Default output  ~/Desktop/Penpot imports          Change   │
│              │  Language        English        More languages are planned. │
│              │  Privacy         No telemetry, no uploads, no network.      │
│              │  About           Version 0.1.x · Licenses                   │
└──────────────┴─────────────────────────────────────────────────────────────┘
```

## Component gallery (dev only)

```text
Buttons: Primary  Secondary  Ghost  Danger
Chips:   ✓ Converted   ⚠ Needs review   ✕ Couldn’t convert   ◼ Cancelled
Rows:    file row (normal / selected / invalid / converting / done)
States:  empty state · banner info/warning/error · progress stages
Type:    title / section / body / caption / monospace
Both themes, long-path and 40%-longer-copy variants.
```
