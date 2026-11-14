import { createFileRoute } from "@tanstack/react-router"
import { useQuery, useQueryClient } from "@tanstack/react-query"
import {
  AlertTriangleIcon,
  BanIcon,
  CheckCircle2Icon,
  CheckIcon,
  CopyIcon,
  CpuIcon,
  KeyRoundIcon,
  PlusIcon,
  RefreshCwIcon,
  RotateCcwIcon,
  ServerIcon,
  ShieldCheckIcon,
  Trash2Icon,
} from "lucide-react"
import { useEffect, useMemo, useState } from "react"

import {
  ApiHttpError,
  authSessionQuery,
  createApiKey,
  deleteApiKey,
  fetchApiKeysList,
  rotateApiKey,
  setApiKeyRevoked,
} from "@/api/styles"
import { AuthGateCard } from "@/components/common/AuthGateCard"
import { PageHeader } from "@/components/common/PageHeader"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { Dialog, DialogContent, DialogHeader, DialogTitle } from "@/components/ui/dialog"
import { Input } from "@/components/ui/input"
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select"
import { toast } from "@/lib/notifications"
import type { PublicApiKeyItem } from "@/types/styles"

export const Route = createFileRoute("/keys")({
  staticData: { title: "API Keys & MCP Access" },
  component: ApiKeysPage,
})

interface McpInfo {
  endpoint: string
  namespace: string
  transport: string
  auth_required: boolean
  tools: { name: string; description: string }[]
  tools_count: number
}

function formatMaskedToken(item: PublicApiKeyItem): string {
  if (item.masked && !item.masked.includes("undefined")) {
    return item.masked
  }
  const suffix = item.last4 ? item.last4 : ""
  return `${item.prefix}••••••••${suffix}`
}

function ApiKeysPage() {
  const queryClient = useQueryClient()
  const sessionQuery = useQuery(authSessionQuery)
  const session = sessionQuery.data

  const [mcpInfo, setMcpInfo] = useState<McpInfo | null>(null)
  const [statusFilter, setStatusFilter] = useState<"all" | "active" | "revoked">("all")
  const [isCreateOpen, setIsCreateOpen] = useState(false)
  const [label, setLabel] = useState("")
  const [customSecret, setCustomSecret] = useState("")
  const [createdSecret, setCreatedSecret] = useState<string | null>(null)
  const [copiedSlot, setCopiedSlot] = useState<string | null>(null)
  const [submitting, setSubmitting] = useState(false)
  const [rotateDialog, setRotateDialog] = useState<{
    open: boolean
    keyId: string
    label: string
    newKey?: string
    rotating?: boolean
  } | null>(null)

  const mustSignIn = Boolean(session?.auth_required && !session?.authenticated)

  const keysQuery = useQuery({
    queryKey: ["api-keys", session?.authenticated, session?.session_epoch],
    queryFn: fetchApiKeysList,
    enabled: !mustSignIn,
    retry: false,
  })

  useEffect(() => {
    let active = true
    fetch("/api/mcp/info", { credentials: "same-origin" })
      .then((res) => (res.ok ? res.json() : null))
      .then((data) => {
        if (active && data) setMcpInfo(data as McpInfo)
      })
      .catch(() => {
        /* non-fatal */
      })
    return () => {
      active = false
    }
  }, [])

  const isUnauthorizedError =
    keysQuery.error instanceof ApiHttpError && keysQuery.error.status === 401

  const keys = keysQuery.data?.keys ?? []
  const filteredKeys = useMemo(() => {
    if (statusFilter === "active") return keys.filter((k) => k.enabled)
    if (statusFilter === "revoked") return keys.filter((k) => !k.enabled)
    return keys
  }, [keys, statusFilter])

  const copyOnceSecret = (secret: string, slot: string) => {
    navigator.clipboard.writeText(secret)
    setCopiedSlot(slot)
    toast.success("Token copied to clipboard")
    setTimeout(() => setCopiedSlot(null), 2000)
  }

  const closeCreateModal = () => {
    setIsCreateOpen(false)
    setCreatedSecret(null)
    setCustomSecret("")
    setLabel("")
  }

  const closeRotateModal = () => {
    setRotateDialog(null)
  }

  const handleCreate = async (e: React.FormEvent) => {
    e.preventDefault()
    if (!label.trim()) {
      toast.error("Please provide a key name or label")
      return
    }
    setSubmitting(true)
    try {
      const data = await createApiKey({
        label: label.trim(),
        custom_secret: customSecret.trim() || undefined,
      })
      setCreatedSecret(data.key)
      setLabel("")
      setCustomSecret("")
      toast.success("API key created — copy it now (shown only once)")
      await queryClient.invalidateQueries({ queryKey: ["api-keys"] })
    } catch (err) {
      toast.error(err instanceof Error ? err.message : "Failed to create key")
    } finally {
      setSubmitting(false)
    }
  }

  const handleRevoke = async (keyId: string, revoke: boolean) => {
    try {
      await setApiKeyRevoked(keyId, revoke)
      toast.success(revoke ? "API key revoked" : "API key re-enabled")
      await queryClient.invalidateQueries({ queryKey: ["api-keys"] })
    } catch (err) {
      toast.error(err instanceof Error ? err.message : "Failed to update key")
    }
  }

  const handleExecuteRotate = async () => {
    if (!rotateDialog) return
    setRotateDialog((prev) => (prev ? { ...prev, rotating: true } : null))
    try {
      const data = await rotateApiKey(rotateDialog.keyId)
      setRotateDialog((prev) =>
        prev ? { ...prev, rotating: false, newKey: data.key } : null
      )
      toast.success(`Key "${rotateDialog.label}" rotated — copy the new secret now`)
      await queryClient.invalidateQueries({ queryKey: ["api-keys"] })
    } catch (err) {
      toast.error(err instanceof Error ? err.message : "Failed to rotate key")
      setRotateDialog((prev) => (prev ? { ...prev, rotating: false } : null))
    }
  }

  const handleDelete = async (keyId: string, keyLabel: string) => {
    if (
      !window.confirm(
        `Permanently DELETE key "${keyLabel}"? Any connected agents using this token will fail immediately.`
      )
    ) {
      return
    }
    try {
      await deleteApiKey(keyId)
      toast.success(`Key "${keyLabel}" permanently deleted`)
      await queryClient.invalidateQueries({ queryKey: ["api-keys"] })
    } catch (err) {
      toast.error(err instanceof Error ? err.message : "Failed to delete key")
    }
  }

  const endpoint = mcpInfo?.endpoint ?? "https://styles.lengoc.me/mcp"
  const namespace = mcpInfo?.namespace ?? "styles__*"

  if (mustSignIn || isUnauthorizedError) {
    return (
      <div className="flex flex-col gap-6">
        <PageHeader
          title="API Keys & MCP Access"
          description="Authenticate via LNC SSO or break-glass management key to issue, rotate, or revoke MCP Bearer tokens."
        />
        <AuthGateCard
          nextPath="/keys"
          oidcEnabled={session?.oidc_enabled ?? true}
          mgmtKeyConfigured={session?.mgmt_key_configured ?? true}
          onAuthenticated={() => {
            queryClient.invalidateQueries({ queryKey: ["api-keys"] })
          }}
        />
      </div>
    )
  }

  return (
    <div className="flex flex-col gap-6">
      <PageHeader
        title="API Keys & MCP Access"
        description="Manage HMAC-hashed Bearer tokens for external agents, Streamable HTTP MCP clients, and Business MCP Hub. Raw secrets are shown once at creation or rotation and never stored in plaintext."
        actions={
          <Button onClick={() => setIsCreateOpen(true)} className="gap-1.5">
            <PlusIcon className="size-4" />
            <span>Generate New Key</span>
          </Button>
        }
      />

      {/* Overview Cards */}
      <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
        <div className="rounded-xl border bg-card p-4 shadow-xs">
          <div className="flex items-center gap-2 text-muted-foreground">
            <ServerIcon className="size-4 text-primary" />
            <span className="text-xs font-semibold tracking-wider uppercase">MCP Endpoint</span>
          </div>
          <div className="mt-2 truncate font-mono text-sm font-semibold text-foreground select-all">
            {endpoint}
          </div>
          <p className="mt-1 text-xs text-muted-foreground">
            {mcpInfo?.transport ?? "Streamable HTTP JSON-RPC 2.0"}
          </p>
        </div>

        <div className="rounded-xl border bg-card p-4 shadow-xs">
          <div className="flex items-center gap-2 text-muted-foreground">
            <ShieldCheckIcon className="size-4 text-emerald-500" />
            <span className="text-xs font-semibold tracking-wider uppercase">Token Storage</span>
          </div>
          <div className="mt-2 text-sm font-semibold text-foreground">
            {mcpInfo?.auth_required ? "Bearer Gate Enforced" : "Salted HMAC-SHA256 Vault"}
          </div>
          <p className="mt-1 text-xs text-muted-foreground">
            Zero-F12-Leak: only key prefix + last 4 chars are retained after creation
          </p>
        </div>

        <div className="rounded-xl border bg-card p-4 shadow-xs">
          <div className="flex items-center gap-2 text-muted-foreground">
            <CpuIcon className="size-4 text-blue-500" />
            <span className="text-xs font-semibold tracking-wider uppercase">Business MCP Hub</span>
          </div>
          <div className="mt-2 text-sm font-semibold text-foreground">
            Online · {mcpInfo?.tools_count ?? 8} Tools Active
          </div>
          <p className="mt-1 text-xs text-muted-foreground">
            Exposes {namespace} tools across fleet agents
          </p>
        </div>
      </div>

      {/* Keys List */}
      <div className="overflow-hidden rounded-2xl border bg-card shadow-xs">
        <div className="flex flex-wrap items-center justify-between gap-2 border-b bg-muted/30 px-5 py-3.5">
          <div className="flex items-center gap-2">
            <KeyRoundIcon className="size-4 text-primary" />
            <h3 className="text-sm font-semibold">Issued API Tokens</h3>
            <Badge variant="secondary" className="font-mono text-[11px]">
              {filteredKeys.length}
            </Badge>
          </div>
          <div className="flex items-center gap-2">
            <Select
              value={statusFilter}
              onValueChange={(v) => setStatusFilter(v as "all" | "active" | "revoked")}
            >
              <SelectTrigger size="sm" className="w-[148px] text-xs">
                <SelectValue placeholder="Filter status" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="all">All Tokens</SelectItem>
                <SelectItem value="active">Active Only</SelectItem>
                <SelectItem value="revoked">Revoked Only</SelectItem>
              </SelectContent>
            </Select>

            <Button
              variant="ghost"
              size="sm"
              onClick={() => keysQuery.refetch()}
              className="h-8 gap-1 text-xs text-muted-foreground"
            >
              <RotateCcwIcon className="size-3.5" />
              <span>Refresh</span>
            </Button>
          </div>
        </div>

        <div className="divide-y">
          {keysQuery.isLoading ? (
            <div className="p-8 text-center text-sm text-muted-foreground">
              Loading API keys...
            </div>
          ) : filteredKeys.length === 0 ? (
            <div className="p-8 text-center text-sm text-muted-foreground">
              No matching API keys found. Click "Generate New Key" to issue a token.
            </div>
          ) : (
            filteredKeys.map((k) => (
              <div
                key={k.id}
                className="flex flex-col justify-between gap-4 p-4 lg:flex-row lg:items-center"
              >
                <div className="flex min-w-0 flex-1 flex-col gap-1.5">
                  <div className="flex flex-wrap items-center gap-2">
                    <span className="text-sm font-semibold text-foreground">{k.label}</span>
                    <Badge
                      variant={k.enabled ? "outline" : "secondary"}
                      className={`text-[10px] ${
                        k.enabled
                          ? "border-emerald-500/40 bg-emerald-500/10 text-emerald-600 dark:text-emerald-400"
                          : "border-destructive/30 bg-destructive/10 text-destructive"
                      }`}
                    >
                      {k.enabled ? "Active" : "Revoked / Disabled"}
                    </Badge>
                  </div>

                  <div className="flex flex-wrap items-center gap-2 text-xs text-muted-foreground">
                    <span className="text-xs font-medium">Identifier:</span>
                    <code className="inline-flex items-center rounded-md border bg-muted/80 px-2 py-0.5 font-mono text-xs tracking-wide text-foreground">
                      {formatMaskedToken(k)}
                    </code>
                    {k.created ? (
                      <span className="ml-1 text-[11px] text-muted-foreground">
                        Created: {new Date(k.created).toLocaleDateString()}
                      </span>
                    ) : null}
                    {k.rotated_at ? (
                      <span className="text-[11px] text-muted-foreground">
                        · Rotated: {new Date(k.rotated_at).toLocaleDateString()}
                      </span>
                    ) : null}
                  </div>
                </div>

                <div className="flex shrink-0 flex-wrap items-center gap-1.5 self-start lg:self-center">
                  <Button
                    variant="outline"
                    size="sm"
                    className="h-8 gap-1 text-xs text-blue-600 hover:bg-blue-50 dark:text-blue-400 dark:hover:bg-blue-950/40"
                    onClick={() => setRotateDialog({ open: true, keyId: k.id, label: k.label })}
                    title="Rotate key and issue a new one-time token"
                  >
                    <RefreshCwIcon className="size-3.5" />
                    <span>Rotate</span>
                  </Button>
                  {k.enabled ? (
                    <Button
                      variant="outline"
                      size="sm"
                      className="h-8 gap-1 text-xs text-amber-600 hover:bg-amber-50 dark:text-amber-400 dark:hover:bg-amber-950/40"
                      onClick={() => handleRevoke(k.id, true)}
                      title="Deactivate this key"
                    >
                      <BanIcon className="size-3.5" />
                      <span>Revoke</span>
                    </Button>
                  ) : (
                    <Button
                      variant="outline"
                      size="sm"
                      className="h-8 gap-1 text-xs text-emerald-600 hover:bg-emerald-50 dark:text-emerald-400 dark:hover:bg-emerald-950/40"
                      onClick={() => handleRevoke(k.id, false)}
                      title="Reactivate this key"
                    >
                      <CheckCircle2Icon className="size-3.5" />
                      <span>Enable</span>
                    </Button>
                  )}
                  <Button
                    variant="outline"
                    size="sm"
                    className="h-8 gap-1 text-xs text-destructive hover:bg-destructive/10 hover:text-destructive"
                    onClick={() => handleDelete(k.id, k.label)}
                    title="Permanently remove key"
                  >
                    <Trash2Icon className="size-3.5" />
                    <span>Delete</span>
                  </Button>
                </div>
              </div>
            ))
          )}
        </div>
      </div>

      {/* Integration Guide */}
      <div className="flex flex-col gap-4 rounded-2xl border bg-muted/40 p-5 text-xs">
        <h3 className="text-sm font-semibold text-foreground">
          How to Connect from AI Agents & MCP Clients
        </h3>
        <div className="grid gap-4 md:grid-cols-2">
          <div className="flex flex-col gap-2 rounded-xl border bg-card p-4">
            <h4 className="text-xs font-semibold text-foreground">
              1. Central Business MCP Hub Registration
            </h4>
            <p className="leading-relaxed text-muted-foreground">
              Connected into the central Business MCP Hub exposing tools with prefix{" "}
              <code className="font-semibold text-foreground">{namespace}</code>.
            </p>
            <pre className="overflow-x-auto rounded-lg bg-neutral-950 p-3 font-mono text-[11px] leading-relaxed text-neutral-300">
              {`# Connect directly to Streamable HTTP MCP
POST ${endpoint}
Headers:
  Authorization: Bearer <YOUR_API_KEY>
  Content-Type: application/json
  Accept: application/json, text/event-stream`}
            </pre>
          </div>

          <div className="flex flex-col gap-2 rounded-xl border bg-card p-4">
            <h4 className="text-xs font-semibold text-foreground">2. Available MCP Tools</h4>
            <ul className="list-disc space-y-1 pl-4 text-muted-foreground">
              {(mcpInfo?.tools ?? []).map((t) => (
                <li key={t.name}>
                  <code className="font-semibold text-foreground">styles__{t.name}</code>:{" "}
                  {t.description.split(".")[0]}.
                </li>
              ))}
              {!mcpInfo ? <li>Loading tool list…</li> : null}
            </ul>
          </div>
        </div>
      </div>

      {/* Create Key Dialog (one-time secret display) */}
      <Dialog
        open={isCreateOpen}
        onOpenChange={(o) => {
          if (!o) closeCreateModal()
        }}
      >
        <DialogContent className="max-w-md">
          <DialogHeader>
            <DialogTitle>Generate New API Key</DialogTitle>
          </DialogHeader>
          {createdSecret ? (
            <div className="my-2 flex flex-col gap-4">
              <div className="rounded-lg border border-amber-500/40 bg-amber-500/10 p-3 text-xs text-amber-800 dark:text-amber-300">
                <strong>Shown once only:</strong> Copy and store this token in your vault now. For security, only its salted HMAC-SHA256 digest is persisted on the server.
              </div>
              <div className="flex items-center gap-2 rounded-xl bg-muted p-3 font-mono text-xs break-all">
                <span className="flex-1 font-semibold text-foreground select-all">
                  {createdSecret}
                </span>
                <Button
                  size="sm"
                  variant="outline"
                  className="h-8 shrink-0 gap-1 text-xs"
                  onClick={() => copyOnceSecret(createdSecret, "newly_created")}
                >
                  {copiedSlot === "newly_created" ? (
                    <CheckIcon className="size-3.5 text-emerald-500" />
                  ) : (
                    <CopyIcon className="size-3.5" />
                  )}
                  <span>Copy</span>
                </Button>
              </div>
              <Button onClick={closeCreateModal} className="w-full">
                I Have Saved This Token
              </Button>
            </div>
          ) : (
            <form onSubmit={handleCreate} className="my-2 flex flex-col gap-4">
              <div className="flex flex-col gap-1.5">
                <label className="text-xs font-semibold text-foreground">
                  Key Name / Client Label
                </label>
                <Input
                  value={label}
                  onChange={(e) => setLabel(e.target.value)}
                  placeholder="e.g. Business MCP Hub, Agent Hermes"
                  required
                />
              </div>
              <div className="flex flex-col gap-1.5">
                <label className="text-xs font-semibold text-foreground">
                  Custom Secret (Optional, min 16 chars)
                </label>
                <Input
                  type="password"
                  value={customSecret}
                  onChange={(e) => setCustomSecret(e.target.value)}
                  placeholder="Leave empty to auto-generate a CSPRNG token"
                />
                <span className="text-[11px] text-muted-foreground">
                  If empty, a 256-bit CSPRNG token starting with{" "}
                  <code className="font-mono">styles_...</code> is generated.
                </span>
              </div>
              <div className="mt-2 flex justify-end gap-2">
                <Button type="button" variant="outline" onClick={closeCreateModal}>
                  Cancel
                </Button>
                <Button type="submit" disabled={submitting}>
                  {submitting ? "Generating..." : "Generate API Key"}
                </Button>
              </div>
            </form>
          )}
        </DialogContent>
      </Dialog>

      {/* Rotate Key Dialog (one-time secret display) */}
      <Dialog
        open={Boolean(rotateDialog?.open)}
        onOpenChange={(o) => {
          if (!o) closeRotateModal()
        }}
      >
        <DialogContent className="max-w-md">
          <DialogHeader>
            <DialogTitle>Rotate API Key</DialogTitle>
          </DialogHeader>
          {rotateDialog?.newKey ? (
            <div className="my-2 flex flex-col gap-4">
              <div className="rounded-lg border border-amber-500/40 bg-amber-500/10 p-3 text-xs text-amber-800 dark:text-amber-300">
                <strong>Shown once only:</strong> Copy this replacement token now. The previous secret has been permanently invalidated.
              </div>
              <div className="flex items-center gap-2 rounded-xl bg-muted p-3 font-mono text-xs break-all">
                <span className="flex-1 font-semibold text-foreground select-all">
                  {rotateDialog.newKey}
                </span>
                <Button
                  size="sm"
                  variant="outline"
                  className="h-8 shrink-0 gap-1 text-xs"
                  onClick={() => copyOnceSecret(rotateDialog.newKey!, "rotated_key")}
                >
                  {copiedSlot === "rotated_key" ? (
                    <CheckIcon className="size-3.5 text-emerald-500" />
                  ) : (
                    <CopyIcon className="size-3.5" />
                  )}
                  <span>Copy</span>
                </Button>
              </div>
              <Button onClick={closeRotateModal} className="w-full">
                I Have Saved This Token
              </Button>
            </div>
          ) : (
            <div className="my-2 flex flex-col gap-4">
              <div className="flex items-start gap-3 rounded-lg border border-amber-500/30 bg-amber-500/10 p-3 text-xs text-amber-800 dark:text-amber-300">
                <AlertTriangleIcon className="mt-0.5 size-4 shrink-0 text-amber-600" />
                <span>
                  Rotating key{" "}
                  <strong className="font-semibold">"{rotateDialog?.label}"</strong> will
                  invalidate the existing secret immediately. Any client or agent using the
                  previous token must be updated.
                </span>
              </div>
              <div className="mt-2 flex justify-end gap-2">
                <Button type="button" variant="outline" onClick={closeRotateModal}>
                  Cancel
                </Button>
                <Button
                  variant="default"
                  onClick={handleExecuteRotate}
                  disabled={rotateDialog?.rotating}
                  className="gap-1.5 bg-blue-600 text-white hover:bg-blue-700"
                >
                  <RefreshCwIcon
                    className={`size-3.5 ${rotateDialog?.rotating ? "animate-spin" : ""}`}
                  />
                  <span>{rotateDialog?.rotating ? "Rotating..." : "Rotate Key Now"}</span>
                </Button>
              </div>
            </div>
          )}
        </DialogContent>
      </Dialog>
    </div>
  )
}
