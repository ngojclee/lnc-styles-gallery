import { useState } from "react"
import { useQueryClient } from "@tanstack/react-query"
import { KeyRound, Lock, ShieldCheck } from "lucide-react"
import { toast } from "sonner"

import { loginWithManagementKey } from "@/api/styles"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"

interface AuthGateCardProps {
  nextPath?: string
  oidcEnabled?: boolean
  mgmtKeyConfigured?: boolean
  compact?: boolean
  onAuthenticated?: () => void
}

export function AuthGateCard({
  nextPath = "/keys",
  oidcEnabled = true,
  mgmtKeyConfigured = true,
  compact = false,
  onAuthenticated,
}: AuthGateCardProps) {
  const queryClient = useQueryClient()
  const [mgmtKey, setMgmtKey] = useState("")
  const [submitting, setSubmitting] = useState(false)
  const [errorMsg, setErrorMsg] = useState<string | null>(null)

  const safeNext = nextPath.startsWith("/") && !nextPath.startsWith("//") ? nextPath : "/keys"
  const oidcHref = `/api/auth/oidc/login?next=${encodeURIComponent(safeNext)}`

  const handleBreakGlassSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    const candidate = mgmtKey.trim()
    if (!candidate) {
      setErrorMsg("Enter a management key to authenticate.")
      return
    }
    setSubmitting(true)
    setErrorMsg(null)
    try {
      await loginWithManagementKey(candidate)
      setMgmtKey("")
      await queryClient.invalidateQueries()
      toast.success("Authenticated with management session")
      onAuthenticated?.()
    } catch (err) {
      setMgmtKey("")
      const msg = err instanceof Error ? err.message : "Authentication failed"
      setErrorMsg(msg)
      toast.error(msg)
    } finally {
      setSubmitting(false)
    }
  }

  return (
    <Card className={compact ? "border-0 shadow-none bg-transparent" : "mx-auto max-w-md border-border/70 shadow-sm"}>
      <CardHeader className={compact ? "px-0 pt-0" : undefined}>
        <div className="mb-1 flex items-center gap-2 text-primary">
          <Lock className="size-4" />
          <span className="text-xs font-semibold tracking-wider uppercase">Protected Console</span>
        </div>
        <CardTitle className="text-lg">Sign in to LNC Style Gallery</CardTitle>
        <CardDescription className="text-xs leading-relaxed">
          API token management and runtime administration require an authenticated LNC SSO session or break-glass management key.
        </CardDescription>
      </CardHeader>

      <CardContent className={compact ? "space-y-4 px-0 pb-0" : "space-y-4"}>
        {errorMsg ? (
          <div className="rounded-lg border border-destructive/40 bg-destructive/10 px-3 py-2 text-xs text-destructive">
            {errorMsg}
          </div>
        ) : null}

        {oidcEnabled ? (
          <Button asChild className="w-full gap-2 font-semibold" size="default">
            <a href={oidcHref}>
              <ShieldCheck className="size-4" />
              <span>Sign in with LNC SSO</span>
            </a>
          </Button>
        ) : (
          <div className="rounded-lg border border-border/60 bg-muted/30 px-3 py-2 text-xs text-muted-foreground">
            OIDC SSO is not configured in this environment. Use the break-glass management key below.
          </div>
        )}

        {mgmtKeyConfigured || !oidcEnabled ? (
          <details
            open={!oidcEnabled}
            className="group rounded-xl border border-border/70 bg-muted/20 px-3.5 py-2.5 text-xs"
          >
            <summary className="flex cursor-pointer list-none items-center justify-between font-medium text-muted-foreground select-none hover:text-foreground">
              <span className="inline-flex items-center gap-1.5">
                <KeyRound className="size-3.5" />
                <span>Break-glass Management Key</span>
              </span>
              <span className="text-[10px] tracking-wider uppercase opacity-70 group-open:rotate-180">▾</span>
            </summary>

            <form onSubmit={handleBreakGlassSubmit} className="mt-3 space-y-3 border-t border-border/50 pt-3">
              <div className="space-y-1.5">
                <Label htmlFor="mgmt-breakglass-key" className="text-xs">
                  Management Key
                </Label>
                <Input
                  id="mgmt-breakglass-key"
                  type="password"
                  autoComplete="current-password"
                  value={mgmtKey}
                  onChange={(e) => setMgmtKey(e.target.value)}
                  placeholder="Enter break-glass management key"
                  className="text-xs"
                />
              </div>
              <Button type="submit" variant="secondary" size="sm" className="w-full" disabled={submitting}>
                {submitting ? "Verifying..." : "Unlock Session"}
              </Button>
            </form>
          </details>
        ) : null}
      </CardContent>
    </Card>
  )
}
