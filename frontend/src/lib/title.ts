// The application's name, which ends every tab title. The server titles a
// page that sets no title of its own with the same name
// (`.default_title("Nebula")` in `src/bootstrap.rs`); change both together.
export const appName = 'Nebula'

/** The tab title for a page titled `title`: `Login · Nebula`, or `Nebula`. */
export function pageTitle(title: string): string {
  return title ? `${title} · ${appName}` : appName
}
