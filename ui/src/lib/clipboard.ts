/**
 * Copy text using Clipboard API when available in a secure context, falling back
 * transparently to a hidden textarea + `document.execCommand("copy")` on LAN HTTP.
 */
export async function copyText(
  value: string,
  onFallbackReveal?: (val: string) => void
): Promise<boolean> {
  if (!value) return false
  try {
    if (navigator.clipboard && window.isSecureContext) {
      await navigator.clipboard.writeText(value)
      return true
    }
  } catch {
    // Fall through to execCommand fallback for non-HTTPS LAN origins
  }
  const ta = document.createElement("textarea")
  ta.value = value
  ta.setAttribute("readonly", "")
  ta.style.position = "fixed"
  ta.style.top = "-9999px"
  ta.style.left = "-9999px"
  ta.style.opacity = "0"
  document.body.appendChild(ta)
  ta.focus()
  ta.select()
  ta.setSelectionRange(0, value.length)
  let ok = false
  try {
    ok = document.execCommand("copy")
  } catch {
    ok = false
  } finally {
    document.body.removeChild(ta)
  }
  if (!ok && onFallbackReveal) {
    onFallbackReveal(value)
  }
  return ok
}
