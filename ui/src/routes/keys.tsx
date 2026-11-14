import { useMemo, useState } from "react"
import { Link, createFileRoute } from "@tanstack/react-router"
import { useQuery, useQueryClient } from "@tanstack/react-query"
import {
  AlertTriangle,
  ArrowRight,
  Check,
  Copy,
  KeyRound,
  Pencil,
  Plug,
  Plus,
  RefreshCw,
  RotateCw,
  Search,
  ShieldCheck,
  ShieldOff,
  Trash2,
} from "lucide-react"
import { toast } from "sonner"

import {
  ApiHttpError,
  authSessionQuery,
  copyApiKey,
  createApiKey,
  deleteApiKey,
  fetchApiKeysList,
  rotateApiKey,
  setApiKeyRevoked,
  updateApiKey,
} from "@/api/styles"
import { AuthGateCard } from "@/components/common/AuthGateCard"
import { PageHeader } from "@/components/common/PageHeader"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select"
import { copyText } from "@/lib/clipboard"
import { cn } from "@/lib/utils"
import type { PublicApiKeyItem } from "@/types/styles"

export const Route = createFileRoute("/keys")({
  component: KeysPage,
})

const SCOPE_PRESETS: Array<{ value: string; label: string; scopes: string[] }> = [
  {
    value: "read",
    label: "Read Catalog & MCP (mcp:read)",
    scopes: ["mcp:read"],
  },
  {
    value: "read_admin",
    label: "Read + Reindex (mcp:read, admin:reindex)",
    scopes: ["mcp:read", "admin:reindex"],
  },
]

function presetValueFromScopes(scopes?: string[]): string {
  if (scopes?.includes("admin:reindex")) return "read_admin"
  return "read"
}

function formatDate(ts?: string | null): string {
  if (!ts) return "—"
  const parsed = new Date(ts)
  if (Number.isNaN(parsed.getTime())) return ts
  return parsed.toLocaleString(undefined, {
    year: "numeric",
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  })
}

export function KeysPage() {
  const queryClient = useQueryClient()
  const sessionQuery = useQuery(authSessionQuery)
  const session = sessionQuery.data
  const mustSignIn = Boolean(session?.auth_required && !session?.authenticated)

  const keysQuery = useQuery({
    queryKey: ["api-keys", session?.authenticated, session?.session_epoch],
    queryFn: fetchApiKeysList,
    enabled: Boolean(session && !mustSignIn),
    retry: false,
  })

  const [label, setLabel] = useState("")
  const [scopePreset, setScopePreset] = useState("read")
  const [customSecret, setCustomSecret] = useState("")
  const [showCustomSecret, setShowCustomSecret] = useState(false)
  const [creating, setCreating] = useState(false)

  const [revealedSecret, setRevealedSecret] = useState<string | null>(null)
  const [revealedLabel, setRevealedLabel] = useState<string>("")
  const [fallbackSecret, setFallbackSecret] = useState<{ label: string; value: string } | null>(null)
  const [copyingId, setCopyingId] = useState<string | null>(null)
  const [copiedBanner, setCopiedBanner] = useState(false)

  const [searchQuery, setSearchQuery] = useState("")
  const [statusFilter, setStatusFilter] = useState<"all" | "active" | "revoked">("all")

  const [renameTarget, setRenameTarget] = useState<PublicApiKeyItem | null>(null)
  const [renameLabel, setRenameLabel] = useState("")
  const [renameScopePreset, setRenameScopePreset] = useState("read")
  const [savingRename, setSavingRename] = useState(false)

  const [rotateTarget, setRotateTarget] = useState<PublicApiKeyItem | null>(null)
  const [rotating, setRotating] = useState(false)

  const [revokeTarget, setRevokeTarget] = useState<PublicApiKeyItem | null>(null)
  const [togglingRevoke, setTogglingRevoke] = useState(false)

  const [deleteTarget, setDeleteTarget] = useState<PublicApiKeyItem | null>(null)
  const [deleting, setDeleting] = useState(false)

  const isUnauthorized =
    mustSignIn ||
    (keysQuery.error instanceof ApiHttpError && keysQuery.error.status === 401)

  const keys = keysQuery.data?.keys ?? []
  const filteredKeys = useMemo(() => {
    const q = searchQuery.trim().toLowerCase()
    return keys.filter((item) => {
      if (statusFilter === "active" && !item.enabled) return false
      if (statusFilter === "revoked" && item.enabled) return false
      if (!q) return true
      return (
        item.label.toLowerCase().includes(q) ||
        item.id.toLowerCase().includes(q) ||
        item.prefix.toLowerCase().includes(q)
      )
    })
  }, [keys, searchQuery, statusFilter])

  const activeCount = keys.filter((k) => k.enabled).length
  const revokedCount = keys.length - activeCount
  const vaultCount = keys.filter((k) => k.copy_available).length

  if (isUnauthorized) {
    return (
      <main className="flex-1 space-y-6">
        <PageHeader
          icon={KeyRound}
          title="API Keys"
          description="Issue, copy, rotate, revoke, and audit bearer keys for REST API and MCP clients."
        />
        <AuthGateCard
          nextPath="/keys"
          oidcEnabled={session?.oidc_enabled}
          mgmtKeyConfigured={session?.mgmt_key_configured}
          onAuthenticated={() => queryClient.invalidateQueries()}
        />
      </main>
    )
  }

  const handleCopyValue = async (secretValue: string, displayLabel: string) => {
    const ok = await copyText(secretValue, (manualValue) => {
      setFallbackSecret({ label: displayLabel, value: manualValue })
    })
    if (ok) {
      toast.success(`Copied key "${displayLabel}" to clipboard`)
    } else {
      toast.info("Clipboard blocked — copy manually from the dialog")
    }
    return ok
  }

  const handleCreate = async (e?: React.FormEvent) => {
    e?.preventDefault()
    const cleanLabel = label.trim()
    if (!cleanLabel) {
      toast.error("Enter a label for the new API key.")
      return
    }
    const preset = SCOPE_PRESETS.find((p) => p.value === scopePreset) ?? SCOPE_PRESETS[0]
    setCreating(true)
    try {
      const res = await createApiKey({
        label: cleanLabel,
        scopes: preset.scopes,
        ...(customSecret.trim() ? { custom_secret: customSecret.trim() } : {}),
      })
      setLabel("")
      setCustomSecret("")
      setShowCustomSecret(false)
      setRevealedSecret(res.key)
      setRevealedLabel(res.item.label)
      await queryClient.invalidateQueries({ queryKey: ["api-keys"] })
      toast.success(`API key "${res.item.label}" issued and sealed in vault`)
    } catch (err) {
      toast.error(err instanceof Error ? err.message : "Failed to create API key")
    } finally {
      setCreating(false)
    }
  }

  const handleVaultCopy = async (item: PublicApiKeyItem) => {
    if (!item.copy_available) {
      toast.info("Legacy hash-only key cannot be copied. Rotate it to enable HKDF vault copy.")
      return
    }
    setCopyingId(item.id)
    try {
      const res = await copyApiKey(item.id)
      await handleCopyValue(res.secret, item.label)
      await queryClient.invalidateQueries({ queryKey: ["admin-settings"] })
    } catch (err) {
      toast.error(err instanceof Error ? err.message : "Copy failed")
    } finally {
      setCopyingId(null)
    }
  }

  const handleSaveRename = async (e?: React.FormEvent) => {
    e?.preventDefault()
    if (!renameTarget) return
    const cleanLabel = renameLabel.trim()
    if (!cleanLabel) {
      toast.error("Label cannot be empty.")
      return
    }
    const preset = SCOPE_PRESETS.find((p) => p.value === renameScopePreset) ?? SCOPE_PRESETS[0]
    setSavingRename(true)
    try {
      await updateApiKey(renameTarget.id, {
        label: cleanLabel,
        scopes: preset.scopes,
      })
      setRenameTarget(null)
      await queryClient.invalidateQueries({ queryKey: ["api-keys"] })
      toast.success(`Updated key "${cleanLabel}"`)
    } catch (err) {
      toast.error(err instanceof Error ? err.message : "Failed to update key")
    } finally {
      setSavingRename(false)
    }
  }

  const handleConfirmRotate = async () => {
    if (!rotateTarget) return
    setRotating(true)
    try {
      const res = await rotateApiKey(rotateTarget.id)
      setRevealedSecret(res.key)
      setRevealedLabel(res.item.label)
      setRotateTarget(null)
      await queryClient.invalidateQueries({ queryKey: ["api-keys"] })
      toast.success(`Rotated "${res.item.label}" — previous secret invalidated`)
    } catch (err) {
      toast.error(err instanceof Error ? err.message : "Failed to rotate API key")
    } finally {
      setRotating(false)
    }
  }

  const handleToggleRevoke = async (item: PublicApiKeyItem, revoke: boolean) => {
    setTogglingRevoke(true)
    try {
      await setApiKeyRevoked(item.id, revoke)
      setRevokeTarget(null)
      await queryClient.invalidateQueries({ queryKey: ["api-keys"] })
      toast.success(
        revoke ? `Revoked key "${item.label}"` : `Re-enabled key "${item.label}"`
      )
    } catch (err) {
      toast.error(err instanceof Error ? err.message : "Failed to update key status")
    } finally {
      setTogglingRevoke(false)
    }
  }

  const handleConfirmDelete = async () => {
    if (!deleteTarget) return
    setDeleting(true)
    try {
      await deleteApiKey(deleteTarget.id)
      const deletedName = deleteTarget.label
      setDeleteTarget(null)
      await queryClient.invalidateQueries({ queryKey: ["api-keys"] })
      toast.success(`Removed API key "${deletedName}"`)
    } catch (err) {
      toast.error(err instanceof Error ? err.message : "Failed to remove API key")
    } finally {
      setDeleting(false)
    }
  }

  return (
    <main className="flex-1 space-y-6">
      <PageHeader
        icon={KeyRound}
        title="API Keys"
        description="Manage HKDF AES-256-GCM vault-backed API keys with Copy · Rename · Rotate · Revoke · Remove."
        actions={
          <>
            <Button asChild variant="outline" size="sm" className="h-7 whitespace-nowrap">
              <Link to="/mcp-connectors">
                <Plug className="size-3.5" />
                <span>MCP Connectors</span>
                <ArrowRight className="size-3.5" />
              </Link>
            </Button>
            <Button
              variant="outline"
              size="sm"
              className="h-7 whitespace-nowrap"
              onClick={() => keysQuery.refetch()}
              disabled={keysQuery.isFetching}
            >
              <RefreshCw className={cn("size-3.5", keysQuery.isFetching && "animate-spin")} />
              <span>Refresh</span>
            </Button>
          </>
        }
      />

      <div className="grid gap-3 sm:grid-cols-3">
        <Card className="border-border/70">
          <CardContent className="flex items-center justify-between py-3.5">
            <div>
              <p className="text-[11px] font-medium tracking-wider text-muted-foreground uppercase">
                Active Keys
              </p>
              <p className="text-xl font-bold tabular-nums">{activeCount}</p>
            </div>
            <Badge variant="outline" className="border-emerald-500/40 bg-emerald-500/10 text-emerald-500">
              {keys.length} total
            </Badge>
          </CardContent>
        </Card>
        <Card className="border-border/70">
          <CardContent className="flex items-center justify-between py-3.5">
            <div>
              <p className="text-[11px] font-medium tracking-wider text-muted-foreground uppercase">
                Vault Sealed
              </p>
              <p className="text-xl font-bold tabular-nums">{vaultCount}</p>
            </div>
            <Badge variant="secondary" className="font-mono text-[11px]">
              AES-256-GCM
            </Badge>
          </CardContent>
        </Card>
        <Card className="border-border/70">
          <CardContent className="flex items-center justify-between py-3.5">
            <div>
              <p className="text-[11px] font-medium tracking-wider text-muted-foreground uppercase">
                Revoked Keys
              </p>
              <p className="text-xl font-bold tabular-nums">{revokedCount}</p>
            </div>
            <Badge variant="outline" className="text-[11px]">
              {revokedCount === 0 ? "Clean" : "Blocked"}
            </Badge>
          </CardContent>
        </Card>
      </div>

      {revealedSecret ? (
        <Card className="border-emerald-500/40 bg-emerald-500/5">
          <CardHeader className="pb-2">
            <div className="flex flex-wrap items-center justify-between gap-2">
              <div className="flex items-center gap-2 text-emerald-500">
                <ShieldCheck className="size-4" />
                <CardTitle className="text-sm font-semibold">
                  Key ready: {revealedLabel}
                </CardTitle>
              </div>
              <Button
                variant="ghost"
                size="sm"
                className="h-7 text-xs"
                onClick={() => {
                  setRevealedSecret(null)
                  setRevealedLabel("")
                }}
              >
                Dismiss
              </Button>
            </div>
            <CardDescription className="text-xs">
              Sealed in HKDF vault storage. Copy now or use the audited Copy action in the table below.
            </CardDescription>
          </CardHeader>
          <CardContent className="flex flex-wrap items-center gap-2">
            <code className="min-w-0 flex-1 overflow-x-auto rounded-lg border border-emerald-500/30 bg-background/90 px-3 py-2 font-mono text-xs text-foreground select-all">
              {revealedSecret}
            </code>
            <Button
              size="sm"
              className="h-7 shrink-0 whitespace-nowrap"
              onClick={async () => {
                const ok = await handleCopyValue(revealedSecret, revealedLabel)
                if (ok) {
                  setCopiedBanner(true)
                  setTimeout(() => setCopiedBanner(false), 1800)
                }
              }}
            >
              {copiedBanner ? <Check className="size-3.5" /> : <Copy className="size-3.5" />}
              <span>{copiedBanner ? "Copied" : "Copy Key"}</span>
            </Button>
          </CardContent>
        </Card>
      ) : null}

      <Card className="border-border/70">
        <CardHeader className="pb-3">
          <CardTitle className="text-base">Issue New API Key</CardTitle>
          <CardDescription className="text-xs">
            Keys are hashed with SHA-256 for Bearer lookup and sealed in HKDF AES-256-GCM vault storage for audited admin copy.
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div
            className="space-y-3"
            onKeyDown={(e) => {
              if (e.key === "Enter" && e.target instanceof HTMLInputElement && !creating) {
                e.preventDefault()
                void handleCreate()
              }
            }}
          >
            <div className="grid gap-3 md:grid-cols-[minmax(0,1fr)_280px_auto]">
              <div className="space-y-1.5">
                <Label htmlFor="new-key-label" className="text-xs">
                  Key Label
                </Label>
                <Input
                  id="new-key-label"
                  value={label}
                  onChange={(e) => setLabel(e.target.value)}
                  placeholder="e.g. local-claude-code"
                  maxLength={80}
                />
              </div>

              <div className="space-y-1.5">
                <Label className="text-xs">Scopes</Label>
                <Select value={scopePreset} onValueChange={setScopePreset}>
                  <SelectTrigger className="w-full" aria-label="New key scopes">
                    <SelectValue placeholder="Select scope preset" />
                  </SelectTrigger>
                  <SelectContent>
                    {SCOPE_PRESETS.map((preset) => (
                      <SelectItem key={preset.value} value={preset.value}>
                        {preset.label}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>

              <div className="flex items-end">
                <Button
                  type="button"
                  disabled={creating}
                  className="w-full md:w-auto"
                  onClick={() => void handleCreate()}
                >
                  <Plus className="size-4" />
                  <span>{creating ? "Issuing..." : "Issue Key"}</span>
                </Button>
              </div>
            </div>

            <div className="space-y-2">
              <button
                type="button"
                className="text-xs font-medium text-muted-foreground underline-offset-4 hover:text-foreground hover:underline"
                onClick={() => setShowCustomSecret((prev) => !prev)}
              >
                {showCustomSecret ? "Hide optional custom key input" : "Use custom secret value (optional)"}
              </button>
              {showCustomSecret ? (
                <div className="space-y-1">
                  <Input
                    type="password"
                    autoComplete="new-password"
                    value={customSecret}
                    onChange={(e) => setCustomSecret(e.target.value)}
                    placeholder="Leave blank to auto-generate lnc_sg_... secret"
                  />
                  <p className="text-[11px] text-muted-foreground">
                    Leave blank for a cryptographically random token (minimum 12 chars if supplied).
                  </p>
                </div>
              ) : null}
            </div>
          </div>
        </CardContent>
      </Card>

      <Card className="border-border/70">
        <CardHeader className="space-y-3 pb-3">
          <div className="flex flex-wrap items-center justify-between gap-2">
            <div>
              <CardTitle className="text-base">Issued API Keys</CardTitle>
              <CardDescription className="text-xs">
                GET endpoints return masked previews only. Copy triggers a rate-limited POST audit event.
              </CardDescription>
            </div>
            <div className="flex w-full flex-wrap items-center gap-2 sm:w-auto">
              <div className="relative min-w-0 flex-1 sm:w-56">
                <Search className="pointer-events-none absolute top-1/2 left-2.5 size-3.5 -translate-y-1/2 text-muted-foreground" />
                <Input
                  value={searchQuery}
                  onChange={(e) => setSearchQuery(e.target.value)}
                  placeholder="Filter keys..."
                  className="h-8 pl-8 text-xs"
                  aria-label="Filter API keys"
                />
              </div>
              <Select
                value={statusFilter}
                onValueChange={(v) => setStatusFilter(v as "all" | "active" | "revoked")}
              >
                <SelectTrigger size="sm" aria-label="Filter key status">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="all">All Statuses</SelectItem>
                  <SelectItem value="active">Active Only</SelectItem>
                  <SelectItem value="revoked">Revoked Only</SelectItem>
                </SelectContent>
              </Select>
            </div>
          </div>
        </CardHeader>

        <CardContent>
          {keysQuery.isLoading ? (
            <p className="py-6 text-center text-xs text-muted-foreground">Loading API keys...</p>
          ) : filteredKeys.length === 0 ? (
            <div className="rounded-xl border border-dashed border-border/70 py-8 text-center">
              <p className="text-sm font-medium">No API keys match the current filter</p>
              <p className="mt-1 text-xs text-muted-foreground">
                Issue a key above or adjust the search/status filter.
              </p>
            </div>
          ) : (
            <div className="divide-y divide-border/60 rounded-xl border border-border/70">
              {filteredKeys.map((item) => (
                <div
                  key={item.id}
                  data-key-row={item.id}
                  className="flex flex-col gap-3 px-4 py-3.5 lg:flex-row lg:items-center lg:justify-between"
                >
                  <div className="min-w-0 space-y-1.5">
                    <div className="flex flex-wrap items-center gap-2">
                      <span className="truncate font-medium text-sm">{item.label}</span>
                      <Badge
                        variant="outline"
                        className={cn(
                          "text-[11px]",
                          item.enabled
                            ? "border-emerald-500/40 bg-emerald-500/10 text-emerald-500"
                            : "border-destructive/40 bg-destructive/10 text-destructive"
                        )}
                      >
                        {item.enabled ? "Active" : "Revoked"}
                      </Badge>
                      <Badge variant="secondary" className="text-[11px]">
                        {item.copy_available ? "Vault Copy" : "Legacy Hash"}
                      </Badge>
                      <code className="rounded bg-muted px-1.5 py-0.5 font-mono text-[11px] text-muted-foreground">
                        {item.masked || `${item.prefix}...${item.last4 ?? "****"}`}
                      </code>
                    </div>

                    <div className="flex flex-wrap items-center gap-x-3 gap-y-1 text-[11px] text-muted-foreground">
                      <span>ID: {item.id}</span>
                      <span>Created: {formatDate(item.created)}</span>
                      {item.rotated_at ? <span>Rotated: {formatDate(item.rotated_at)}</span> : null}
                      {item.revoked_at ? <span>Revoked: {formatDate(item.revoked_at)}</span> : null}
                      <span className="inline-flex flex-wrap items-center gap-1">
                        {(item.scopes ?? ["mcp:read"]).map((scope) => (
                          <Badge key={scope} variant="outline" className="px-1.5 py-0 font-mono text-[10px]">
                            {scope}
                          </Badge>
                        ))}
                      </span>
                    </div>
                  </div>

                  <div
                    data-action-bar="api-key"
                    className="flex flex-wrap items-center gap-1.5 sm:flex-nowrap shrink-0"
                  >
                    <Button
                      variant="outline"
                      size="sm"
                      className="h-7 shrink-0 px-2.5 text-xs whitespace-nowrap"
                      disabled={!item.copy_available || copyingId === item.id}
                      title={
                        item.copy_available
                          ? "Decrypt and copy API key (audited)"
                          : "Legacy hash-only key — rotate to enable encrypted copy"
                      }
                      onClick={() => handleVaultCopy(item)}
                    >
                      <Copy className="size-3.5" />
                      <span>{copyingId === item.id ? "Copying..." : "Copy"}</span>
                    </Button>

                    <Button
                      variant="outline"
                      size="sm"
                      className="h-7 shrink-0 px-2.5 text-xs whitespace-nowrap"
                      onClick={() => {
                        setRenameTarget(item)
                        setRenameLabel(item.label)
                        setRenameScopePreset(presetValueFromScopes(item.scopes))
                      }}
                    >
                      <Pencil className="size-3.5" />
                      <span>Rename</span>
                    </Button>

                    <Button
                      variant="outline"
                      size="sm"
                      className="h-7 shrink-0 px-2.5 text-xs whitespace-nowrap"
                      onClick={() => setRotateTarget(item)}
                    >
                      <RotateCw className="size-3.5" />
                      <span>Rotate</span>
                    </Button>

                    {item.enabled ? (
                      <Button
                        variant="outline"
                        size="sm"
                        className="h-7 shrink-0 px-2.5 text-xs whitespace-nowrap text-amber-600 hover:text-amber-500 dark:text-amber-400"
                        onClick={() => setRevokeTarget(item)}
                      >
                        <ShieldOff className="size-3.5" />
                        <span>Revoke</span>
                      </Button>
                    ) : (
                      <Button
                        variant="outline"
                        size="sm"
                        className="h-7 shrink-0 px-2.5 text-xs whitespace-nowrap text-emerald-600 hover:text-emerald-500 dark:text-emerald-400"
                        disabled={togglingRevoke}
                        onClick={() => handleToggleRevoke(item, false)}
                      >
                        <ShieldCheck className="size-3.5" />
                        <span>Enable</span>
                      </Button>
                    )}

                    <Button
                      variant="outline"
                      size="sm"
                      className="h-7 shrink-0 px-2.5 text-xs whitespace-nowrap text-destructive hover:bg-destructive/10 hover:text-destructive"
                      onClick={() => setDeleteTarget(item)}
                    >
                      <Trash2 className="size-3.5" />
                      <span>Remove</span>
                    </Button>
                  </div>
                </div>
              ))}
            </div>
          )}
        </CardContent>
      </Card>

      {/* Rename & Scope Modal */}
      <Dialog open={Boolean(renameTarget)} onOpenChange={(open) => !open && setRenameTarget(null)}>
        <DialogContent>
          <div
            className="space-y-4"
            onKeyDown={(e) => {
              if (e.key === "Enter" && e.target instanceof HTMLInputElement && !savingRename) {
                e.preventDefault()
                void handleSaveRename()
              }
            }}
          >
            <DialogHeader>
              <DialogTitle>Edit API Key</DialogTitle>
              <DialogDescription>
                Update the key label or assigned capability scopes without changing its secret.
              </DialogDescription>
            </DialogHeader>
            <div className="space-y-3">
              <div className="space-y-1.5">
                <Label htmlFor="rename-key-label" className="text-xs">
                  Label
                </Label>
                <Input
                  id="rename-key-label"
                  value={renameLabel}
                  onChange={(e) => setRenameLabel(e.target.value)}
                  maxLength={80}
                />
              </div>
              <div className="space-y-1.5">
                <Label className="text-xs">Scopes</Label>
                <Select value={renameScopePreset} onValueChange={setRenameScopePreset}>
                  <SelectTrigger className="w-full" aria-label="Edit key scopes">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    {SCOPE_PRESETS.map((preset) => (
                      <SelectItem key={preset.value} value={preset.value}>
                        {preset.label}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
            </div>
            <DialogFooter>
              <Button type="button" variant="outline" onClick={() => setRenameTarget(null)}>
                Cancel
              </Button>
              <Button
                type="button"
                disabled={savingRename}
                onClick={() => void handleSaveRename()}
              >
                {savingRename ? "Saving..." : "Save Changes"}
              </Button>
            </DialogFooter>
          </div>
        </DialogContent>
      </Dialog>

      {/* Rotate Modal */}
      <Dialog open={Boolean(rotateTarget)} onOpenChange={(open) => !open && setRotateTarget(null)}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Rotate API Key?</DialogTitle>
            <DialogDescription>
              Rotating <strong>{rotateTarget?.label}</strong> immediately generates a new vault-sealed token and invalidates the previous secret across all clients.
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button variant="outline" onClick={() => setRotateTarget(null)}>
              Cancel
            </Button>
            <Button onClick={handleConfirmRotate} disabled={rotating}>
              {rotating ? "Rotating..." : "Confirm Rotate"}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Revoke Modal */}
      <Dialog open={Boolean(revokeTarget)} onOpenChange={(open) => !open && setRevokeTarget(null)}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle className="flex items-center gap-2">
              <AlertTriangle className="size-4 text-amber-500" />
              <span>Revoke API Key?</span>
            </DialogTitle>
            <DialogDescription>
              Clients authenticating with <strong>{revokeTarget?.label}</strong> will receive <code>401 Unauthorized</code> immediately. You can re-enable or remove it later.
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button variant="outline" onClick={() => setRevokeTarget(null)}>
              Cancel
            </Button>
            <Button
              variant="destructive"
              disabled={togglingRevoke}
              onClick={() => revokeTarget && handleToggleRevoke(revokeTarget, true)}
            >
              {togglingRevoke ? "Revoking..." : "Revoke Key"}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Remove Modal */}
      <Dialog open={Boolean(deleteTarget)} onOpenChange={(open) => !open && setDeleteTarget(null)}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle className="flex items-center gap-2 text-destructive">
              <Trash2 className="size-4" />
              <span>Remove API Key?</span>
            </DialogTitle>
            <DialogDescription>
              Permanently deletes <strong>{deleteTarget?.label}</strong> and purges its HKDF encrypted vault envelope. Any client still using this token will lose access immediately.
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button variant="outline" onClick={() => setDeleteTarget(null)}>
              Cancel
            </Button>
            <Button variant="destructive" onClick={handleConfirmDelete} disabled={deleting}>
              {deleting ? "Removing..." : "Remove Permanently"}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Manual HTTP Clipboard Fallback Modal */}
      <Dialog
        open={Boolean(fallbackSecret)}
        onOpenChange={(open) => {
          if (!open) setFallbackSecret(null)
        }}
      >
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Manual Copy Required</DialogTitle>
            <DialogDescription>
              Automatic clipboard access was blocked for <strong>{fallbackSecret?.label}</strong>. Select and copy the key below, then close this dialog.
            </DialogDescription>
          </DialogHeader>
          <code className="block overflow-x-auto rounded-lg border bg-muted p-3 font-mono text-xs select-all">
            {fallbackSecret?.value}
          </code>
          <DialogFooter>
            <Button onClick={() => setFallbackSecret(null)}>Done</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </main>
  )
}
