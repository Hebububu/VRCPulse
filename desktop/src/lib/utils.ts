const isTauri = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;

export async function openUrl(url: string) {
  if (isTauri) {
    const { open } = await import('@tauri-apps/plugin-shell');
    await open(url);
  } else {
    window.open(url, '_blank');
  }
}
