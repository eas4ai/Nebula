// Suprnova flashes validation errors as a field -> messages map
// (`HashMap<String, Vec<String>>` on the wire), so every error value the
// Inertia client sees - `page.props.errors` and each `form.errors` - is a
// `string[]`, not Inertia's default `string`. This declaration-merge teaches
// the client types the real shape; pages render the first message via
// `form.errors.field?.[0]`.
//
// If `@inertiajs/core` ever renames `InertiaConfig`, this augmentation
// silently stops merging - the side-effect import below makes `tsc` flag
// that case instead of typing against a phantom module.
import '@inertiajs/core'

declare module '@inertiajs/core' {
  export interface InertiaConfig {
    errorValueType: string[]
    // Shape of `page.flash`: the layout surfaces `success` / `error` copy
    // flashed by Suprnova's `redirect!(...).with(...)` responses.
    flashDataType: { success?: string; error?: string }
  }
}
