# AI Rules

## UI Component Standards

- **ALWAYS use shadcn/ui-style components** from `web/src/react/components/ui/`:
  - `Dialog`, `DialogContent`, `DialogHeader`, `DialogTitle`, `DialogBody`, `DialogFooter` - for modals
  - `Select`, `SelectOption` - for dropdowns (NOT native `<select>`)
  - `Input`, `Textarea` - for text inputs (NOT native `<input>` or `<textarea>`)
  - `Button` - for buttons with variants: `default`, `secondary`, `outline`, `ghost`, `destructive`
  - `Label`, `Hint`, `Warning` - for form labels and helper text

- **DO NOT use native HTML elements** for UI in React components:
  - ❌ `<select>` → ✅ `<Select>`
  - ❌ `<input>` → ✅ `<Input>`
  - ❌ `<textarea>` → ✅ `<Textarea>`
  - ❌ `<button>` → ✅ `<Button>`

- **DO NOT use** `window.alert()`, `window.confirm()`, or `window.prompt()` for user-facing messages.

- Use the shadcn/ui-style `Dialog` component for success/error prompts (e.g., registration success).
