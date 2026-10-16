/**
 * The server sends every validation message for a field as `string[]`
 * (`with_all_errors(true)` in `src/bootstrap.rs`). shadcn's `Field.Error`
 * takes `{ message }[]`; Nebula shows the first message only.
 */
export function fieldError(messages?: string[]): { message: string }[] | undefined {
  const first = messages?.[0]
  return first ? [{ message: first }] : undefined
}
