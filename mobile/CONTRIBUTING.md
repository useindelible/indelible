# Mobile Contributing Guide

> For AI sessions and human contributors working in `mobile/`.  
> Read this **before touching any file** in `mobile/composeApp/`.

---

## Table of Contents

1. [Architecture Overview](#architecture-overview)
2. [Design System — The Rules](#design-system--the-rules)
3. [Colour](#colour)
4. [Typography](#typography)
5. [Spacing & Layout](#spacing--layout)
6. [Shapes & Radius](#shapes--radius)
7. [How to Introduce a New Component](#how-to-introduce-a-new-component)
8. [How to Introduce a New Screen](#how-to-introduce-a-new-screen)
9. [Navigation](#navigation)
10. [State Management](#state-management)
11. [Do's and Don'ts](#dos-and-donts)
12. [Component Checklist](#component-checklist)

---

## Architecture Overview

```
mobile/composeApp/src/commonMain/kotlin/app/indelible/
│
├── ui/
│   ├── theme/           ← THE LAW — touch with extreme care
│   │   ├── AppTheme.kt  ← Root composable — wrap everything here
│   │   ├── Color.kt     ← All colour constants
│   │   ├── Type.kt      ← IndelibleTypography — 11 type roles
│   │   ├── Spacing.kt   ← IndelibleSpacing — 4dp grid constants
│   │   └── Shape.kt     ← IndelibleShape + Material Shapes wiring
│   └── components/      ← Shared, reusable primitive components
│       ├── IndelibleButton.kt
│       └── IndelibleTextField.kt
│
├── auth/
│   ├── ui/
│   │   ├── components/  ← Auth-specific thin wrappers
│   │   ├── LoginScreen.kt
│   │   ├── RegisterScreen.kt
│   │   ├── ForgotPasswordScreen.kt
│   │   └── VerifyEmailScreen.kt
│   ├── viewmodel/
│   └── navigation/
│
├── onboarding/
│   ├── ui/
│   │   ├── components/  ← StepCard, ProviderCard, PagerIndicator
│   │   └── *Step.kt     ← Individual step screens
│   └── viewmodel/
│
├── profile/
│   ├── ui/
│   │   ├── components/  ← SettingsRow, SettingsSection
│   │   └── *Screen.kt
│   └── viewmodel/
│
├── mila/
│   ├── data/            ← MilaRepository, domain models (ChatScope, ChatMessage, StreamEvent)
│   ├── viewmodel/       ← MilaChatViewModel, MilaChatUiState, MilaChatEffect
│   └── ui/              ← MilaChatScreen
│
├── navigation/          ← MainNavigation, TabItem
├── stubs/               ← Placeholder screens — replace before shipping
└── App.kt               ← Root composable — do NOT add logic here
```

---

## Design System — The Rules

The design system lives exclusively in `ui/theme/`. It is the **single source of truth** for all visual decisions. No screen or component is allowed to freestyle colours, type sizes, or spacing.

> [!IMPORTANT]
> **Never** hardcode hex colours, font sizes, or `dp` spacing values directly in a screen or component file. Every value must trace back to a token in `ui/theme/`.

The `AppTheme { }` composable (called once in `App.kt`) injects the entire token system into `MaterialTheme`. All composables lower in the tree access tokens via `MaterialTheme.colorScheme.*`, `MaterialTheme.typography.*`, and `MaterialTheme.shapes.*`. This means you almost never need to import anything from `ui/theme/` except `IndelibleSpacing`.

---

## Colour

### Reference

| `MaterialTheme.colorScheme.*` | Indelible token | Light | Dark | Use for |
|---|---|---|---|---|
| `primary` | accent | `#0071E3` | `#0A84FF` | CTA buttons, links, active states, focus rings |
| `onPrimary` | — | `#FFFFFF` | `#FFFFFF` | Text/icon on `primary` filled area |
| `primaryContainer` | fill-selected | `α8% accent` | `α14% accent` | Selected card background, chip fill |
| `onPrimaryContainer` | accent | same | same | Text on `primaryContainer` |
| `background` | bg-primary | `#FFFFFF` | `#000000` | Root screen background |
| `surface` | bg-primary | `#FFFFFF` | `#000000` | Cards, modals, overlapping layers |
| `onBackground` / `onSurface` | text-primary | `#1D1D1F` | `#F5F5F7` | Headlines, body copy |
| `surfaceVariant` | bg-secondary | `#F5F5F7` | `#1C1C1E` | Grouped areas, filled inputs, sidebars |
| `onSurfaceVariant` | text-secondary | `#86868B` | `#98989D` | Subtitles, metadata, disabled labels |
| `surfaceContainer` | bg-elevated | `#FFFFFF` | `#2C2C2E` | Modals, popovers, floating elements |
| `surfaceContainerHigh` | bg-tertiary | `#E8E8ED` | `#2C2C2E` | Deep-nested grouped rows |
| `outline` | border-secondary | `α10% black` | `α12% white` | Input borders, dividers |
| `outlineVariant` | border-primary | `α7% black` | `α8% white` | Subtle separators |
| `error` | destructive | `#FF3B30` | `#FF453A` | Error states, destructive actions |

### Extended Colors (beyond MaterialTheme)

Some semantic roles don't map to Material3 `ColorScheme` slots. These are exposed via `IndelibleTheme.colors` (backed by `LocalIndelibleColors`).

| `IndelibleTheme.colors.*` | Indelible token | Light | Dark | Use for |
|---|---|---|---|---|
| `warning` | warning | `#FF9500` | `#FF9F0A` | Warning states, "Later" triage action |
| `onWarning` | — | `#FFFFFF` | `#FFFFFF` | Text/icon on `warning` filled area |
| `success` | success | `#34C759` | `#30D158` | Success states, "Archive" triage action |
| `onSuccess` | — | `#FFFFFF` | `#FFFFFF` | Text/icon on `success` filled area |

Access pattern:
```kotlin
import app.indelible.ui.theme.IndelibleTheme

Box(modifier = Modifier.background(IndelibleTheme.colors.warning))
Text(color = IndelibleTheme.colors.onWarning)
```

### Rules

- **Use semantic slots** (`primary`, `onSurface`, etc.) — never use raw `Color.kt` constants inside composables.
- **Never** use `Color(0xFFFF0000)` or similar literals in a composable.
- **Never** use `secondary`, `tertiary`, `secondaryContainer`, or `tertiaryContainer` slots — Indelible doesn't use them. Use `primaryContainer` / `surfaceVariant` instead.
- Alpha/transparency: use `.copy(alpha = ...)` on a semantic colour only when a spec explicitly calls for a percentage — prefer dedicated tokens.

---

## Typography

All type is accessed via `MaterialTheme.typography.*`. The mapping from Indelible design roles to Material slots is:

| Material slot | Indelible role | Size | Weight | Tracking | Use for |
|---|---|---|---|---|---|
| `displaySmall` | display | 34sp | 700 | -0.04em | Empty state hero, onboarding splash |
| `headlineLarge` | title-1 | 28sp | 700 | -0.03em | Page titles (Library, Settings) |
| `headlineMedium` | title-2 | 22sp | 700 | -0.03em | Section headers, panel titles |
| `headlineSmall` | title-3 | 20sp | 600 | -0.025em | Detail view article titles |
| `titleLarge` | headline | 17sp | 600 | -0.02em | List item titles, bold labels |
| `bodyLarge` | body | 15sp | 400 | -0.01em | Primary body text |
| `titleSmall` | callout | 14sp | 600 | -0.01em | Row titles in compact lists |
| `bodyMedium` | subheadline | 13sp | 400 | -0.01em | Metadata, secondary descriptions, nav labels |
| `bodySmall` | footnote | 12sp | 400 | -0.005em | Timestamps, source labels, error messages |
| `labelSmall` | caption-2 | 11sp | 400 | -0.005em | Tab bar labels, tertiary metadata |

**caption-1** (section headers — uppercase + wide tracking) is also `labelSmall`, but with an explicit `.copy()`:

```kotlin
// caption-1: section labels — upstream this if it becomes common
MaterialTheme.typography.labelSmall.copy(
    fontWeight = FontWeight.Medium,
    letterSpacing = 0.06.em,
)
```

### Rules

- **Never** set `fontSize`, `fontWeight`, or `letterSpacing` directly on a `Text()` call. Always use a `style = MaterialTheme.typography.*` slot.
- **Never** use `titleMedium` — it has no Indelible mapping and will produce an incorrect visual result.
- **Never** use `labelLarge`, `labelMedium` — same reason.
- If a new type treatment is needed that doesn't map to an existing slot, **update `Type.kt`** and this document — do not freestyle it at the call site.

---

## Spacing & Layout

All spacing comes from `IndelibleSpacing`. Import it as:

```kotlin
import app.indelible.ui.theme.IndelibleSpacing
```

### Values

| Constant | Value | Use |
|---|---|---|
| `step2` | 2dp | Hairline gaps, indicator stroke width |
| `step4` | 4dp | Inline icon gap, tight padding |
| `step8` | 8dp | Between stacked items of same type |
| `step12` | 12dp | Card inner padding (compact) |
| `step16` | 16dp | Standard content gap, card inner padding |
| `step20` | 20dp | Row icon size, larger icon padding |
| `step24` | 24dp | Screen horizontal padding (`screenPaddingH`) |
| `step32` | 32dp | Screen vertical padding (`screenPaddingV`) |
| `step40` | 40dp | Section breathing room |
| `step48` | 48dp | `touchTarget` — minimum button height |
| `step64` | 64dp | Header/illustration zone |
| `sectionGap` | 24dp | Between major sections on a screen |
| `contentGap` | 16dp | Between stacked items in a card |
| `rowPaddingH` | 20dp | List row horizontal padding |
| `rowPaddingV` | 14dp | List row vertical padding |
| `screenPaddingH` | 24dp | Full-screen card/page horizontal padding |
| `screenPaddingV` | 32dp | Full-screen card/page vertical padding |
| `touchTarget` | 48dp | Minimum interactive element height |

### Rules

- **Never** write `Modifier.padding(16.dp)` — use `IndelibleSpacing.step16`.
- **Never** write `Modifier.height(48.dp)` for a button — use `IndelibleSpacing.touchTarget`.
- **Never** use odd values like `13.dp`, `17.dp`, `22.dp` — all values must be on the 4dp grid.
- Spacer heights follow the same rule: `Spacer(modifier = Modifier.height(IndelibleSpacing.step8))`.

---

## Shapes & Radius

Shapes are applied automatically by Material components once `AppTheme` sets the global `Shapes`. You rarely need to set shapes explicitly, but when you do:

| `MaterialTheme.shapes.*` | `IndelibleShape.*` | Radius | Use |
|---|---|---|---|
| `small` | `sm` | 7dp | Buttons, chips, text fields, segmented controls |
| `medium` | `md` | 10dp | Cards, banners, metadata rows |
| `large` | `lg` | 12dp | Bottom sheets, nav drawer |
| `extraLarge` | `xl` | 14dp | Auth cards, modal overlays |

Use `IndelibleShape.full` (980dp) for pill-shaped elements (e.g., toggle backgrounds).

### Rules

- **Never** set `RoundedCornerShape(8.dp)` or similar literals inline.
- **Always** use `MaterialTheme.shapes.*` or `IndelibleShape.*` from `Shape.kt`.

---

## How to Introduce a New Component

### Step 1 — Decide if it belongs in `ui/components/` or a feature's `components/` folder

| Scenario | Location |
|---|---|
| Used by 2+ features, or clearly reusable | `ui/components/` |
| Used only within a single feature | `featureName/ui/components/` |

### Step 2 — Write the composable

Requirements:
- All colours from `MaterialTheme.colorScheme.*`
- All typography from `MaterialTheme.typography.*`
- All shapes from `MaterialTheme.shapes.*` or `IndelibleShape.*`
- All spacing from `IndelibleSpacing.*`
- `modifier: Modifier = Modifier` as the first named parameter after required props
- User-facing copy belongs in `composeApp/src/commonMain/composeResources/values/strings.xml`; render it with `stringResource` or `pluralStringResource` instead of hardcoding it

Template:

```kotlin
package app.indelible.ui.components   // or feature path

import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import app.indelible.ui.theme.IndelibleSpacing

@Composable
fun MyComponent(
    // required data parameters first
    someValue: String,
    onAction: () -> Unit,
    // modifier last, always defaulting to Modifier
    modifier: Modifier = Modifier,
) {
    // Use MaterialTheme.colorScheme.*, MaterialTheme.typography.*, etc.
    // Use IndelibleSpacing.* for all dp values
}
```

### Step 3 — Write a `@Preview`

Every new component **must** have at least two Previews: one light, one dark.

```kotlin
@Preview(showBackground = true)
@Composable
private fun MyComponentPreviewLight() {
    AppTheme(darkTheme = false) { MyComponent(someValue = "Demo", onAction = {}) }
}

@Preview(showBackground = true, uiMode = UI_MODE_NIGHT_YES)
@Composable
private fun MyComponentPreviewDark() {
    AppTheme(darkTheme = true) { MyComponent(someValue = "Demo", onAction = {}) }
}
```

### Step 4 — Update this document

Add a row to the architecture overview table under the appropriate section.

---

## How to Introduce a New Screen

### Step 1 — Create the screen file

Location: `featureName/ui/FeatureNameScreen.kt`

The composable signature should:
- Accept only data and callbacks (no `ViewModel` injected directly into the composable)
- Accept `viewModel: FeatureViewModel = hiltViewModel()` **only** if the feature has a dedicated ViewModel entry point — otherwise pass state down

### Step 2 — Create or update the ViewModel

Location: `featureName/viewmodel/FeatureViewModel.kt`

- `ViewModel` subclass using `viewModelScope` + coroutines
- Expose state as `StateFlow<FeatureUiState>` (not `LiveData`)
- Define a sealed class `FeatureUiState` with `Loading`, `Success`, `Error` variants

### Step 3 — Wire navigation

For a new tab: add to `TabItem` enum in `MainNavigation.kt`.  
For a detail screen: add a route constant to the relevant `*Routes` object and add a `composable(...)` block to the relevant `NavHost`.

### Step 4 — Use only design system primitives

- Wrap the screen's layout in the appropriate structural component (`StepCard`, a `Scaffold`, etc.)
- Do not introduce a new structural layout pattern without first checking if an existing component can be extended
- Buttons → `IndelibleButton`
- Text inputs → `IndelibleTextField`
- Settings rows → `SettingsRow` / `SettingsSection`

---

## Navigation

- All routes are `String` constants in a `*Routes` object in the relevant `*Navigation.kt` or `MainNavigation.kt`
- **Never** navigate by calling `navController.navigate("hardcoded/string")` — always reference a routes constant
- Deep links TBD — do not implement until the backend route scheme is finalised
- `NavHost` start destination for tabs: always `TabItem.LIBRARY.route`

---

## State Management

- **ViewModel** per feature — never share a ViewModel across unrelated features
- **UI state** modelled as a sealed class or data class passed down as a single `uiState: FeatureUiState` parameter
- **Side effects** (navigation, toasts) emitted as a `SharedFlow<FeatureEffect>`, consumed with `LaunchedEffect` in the screen
- **Never** put navigation calls or context-dependent code inside a ViewModel
- **Never** hold `Context` in a ViewModel — use repositories/use cases insteads

---

## Do's and Don'ts

### ✅ Do

```kotlin
// ✅ Use semantic colour tokens
Text(color = MaterialTheme.colorScheme.onSurfaceVariant)

// ✅ Use typography slots
Text(style = MaterialTheme.typography.bodyMedium)

// ✅ Use spacing constants
Modifier.padding(horizontal = IndelibleSpacing.screenPaddingH)

// ✅ Use shape tokens
Card(shape = MaterialTheme.shapes.medium)

// ✅ Use shared components
IndelibleButton(text = "Save", onClick = onSave)
IndelibleTextField(value = email, onValueChange = onEmailChange, label = "Email")

// ✅ Use AuthButton/AuthTextField wrappers inside auth screens
AuthButton(text = "Sign in", onClick = onSignIn)

// ✅ Preview every component in both modes
@Preview(uiMode = UI_MODE_NIGHT_YES) @Composable private fun DarkPreview() { ... }
```

### ❌ Don't

```kotlin
// ❌ Never hardcode colours
Text(color = Color(0xFF86868B))
Text(color = Color.Gray)

// ❌ Never use unused Material slots
Card(colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.secondaryContainer))
Card(colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.tertiaryContainer))

// ❌ Never hardcode font sizes
Text(fontSize = 13.sp)

// ❌ Never use unlisted typography slots
Text(style = MaterialTheme.typography.titleMedium)  // no Indelible mapping
Text(style = MaterialTheme.typography.labelLarge)   // no Indelible mapping

// ❌ Never hardcode spacing
Modifier.padding(16.dp)
Modifier.height(48.dp)
Spacer(modifier = Modifier.height(24.dp))

// ❌ Never use raw dp for shapes
Card(shape = RoundedCornerShape(12.dp))

// ❌ Never use OutlinedTextField directly — use IndelibleTextField
OutlinedTextField(value = ..., ...)

// ❌ Never use Button directly in a screen — use IndelibleButton
Button(onClick = ...) { Text("Go") }

// ❌ Never inline a loading spinner pattern — it belongs in IndelibleButton
Button(onClick = ...) {
    if (isLoading) CircularProgressIndicator() else Text("Save")
}

// ❌ Never call MaterialTheme {} without AppTheme wrapping it
MaterialTheme { ... }

// ❌ Never add colour constants or spacing values outside ui/theme/
val myPurple = Color(0xFF5856D6)  // wrong — add to Color.kt if genuinely needed
```

---

## Component Checklist

Before opening a PR or concluding a session, verify:

- [ ] No raw `Color(...)` literals anywhere in the diff
- [ ] No raw `.dp` spacing literals anywhere in the diff (only through `IndelibleSpacing.*`)
- [ ] No `fontSize = N.sp` on any `Text()` call
- [ ] No `RoundedCornerShape(N.dp)` literals — only `MaterialTheme.shapes.*` or `IndelibleShape.*`
- [ ] No direct use of `Button`, `OutlinedTextField`, or `MaterialTheme {}` in screen files
- [ ] No `secondaryContainer`, `tertiaryContainer`, `labelLarge`, `labelMedium`, `titleMedium` slots used
- [ ] New component has both light and dark `@Preview`
- [ ] New route strings are constants in a `*Routes` object
- [ ] ViewModel exposes `StateFlow`, not `LiveData`
- [ ] Build passes: `./gradlew :composeApp:compileDebugKotlin`
- [ ] Tests pass: `./gradlew :composeApp:testDebugUnitTest`

---

## Modifying the Design System (`ui/theme/`)

If a design change requires updating token values:

1. Change the value **only** in `ui/theme/Color.kt`, `Type.kt`, `Spacing.kt`, or `Shape.kt`
2. **Never** update a token value at a call site — that defeats the whole system
3. If a new semantic concept is needed (e.g., a new colour role), add it to `AppTheme.kt`'s `lightColorScheme`/`darkColorScheme` and map it to an appropriate Material slot
4. Document the new token in the mapping table in this file and in `ui/theme/AppTheme.kt`'s comments
5. Verify the value against the design tokens in `mobile/composeApp/src/commonMain/kotlin/app/indelible/ui/theme/`

---

## Quick Reference

```
New colour needed?       → Check Color.kt first. Add there if missing.
New type size needed?    → Check Type.kt mapping table. Never freestyle.
New spacing value?       → Check IndelibleSpacing. Only add 4dp-grid values.
New button variant?      → Extend IndelibleButton with a style parameter.
New text field variant?  → Extend IndelibleTextField with a parameter.
New screen?              → ViewModel + UiState + Screen composable + route constant.
New shared component?    → ui/components/ with Previews, tokens only.
```
