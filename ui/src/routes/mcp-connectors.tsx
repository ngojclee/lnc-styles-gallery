import { useState } from "react"
import { Link, createFileRoute } from "@tanstack/react-router"
import { useQuery, useQueryClient } from "@tanstack/react-query"
import {
  Activity,
  AlertTriangle,
  ArrowRight,
  Check,
  Copy,
  KeyRound,
  Lock,
  Pencil,
  Plug,
  Plus,
  RefreshCw,
  RotateCw,
  ShieldCheck,
  ShieldOff,
  Trash2,
  Wrench,
} from "lucide-react"
import { toast } from "sonner"

import {
  ApiHttpError,
  authSessionQuery,
  copyMcpConnector,
  createMcpConnector,
  deleteMcpConnector,
  fetchMcpConnectors,
  rotateMcpConnector,
  setMcpConnectorRevoked,
  testMcpConnector,
  updateMcpConnector,
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
import type { McpConnectorItem, McpConnectorTestResponse } from "@/types/styles"

export const Route = createFileRoute("/mcp-connectors")({
  component: McpConnectorsPage,
})

const CONNECTOR_SCOPE_PRESETS: Array<{ value: string; label: string; scopes: string[] }> = [
  {
    value: "read",
    label: "Read Styles & MCP Tools (mcp:read)",
    scopes: ["mcp:read"],
  },
  {
    value: "read_admin",
    label: "Read + Reindex Corpus (mcp:read, admin:reindex)",
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

export function McpConnectorsPage() {
  const queryClient = useQueryClient()
  const sessionQuery = useQuery(authSessionQuery)
  const session = sessionQuery.data
  const mustSignIn = Boolean(session?.auth_required && !session?.authenticated)

  const connectorsQuery = useQuery({
    queryKey: ["mcp-connectors", session?.authenticated, session?.session_epoch],
    queryFn: fetchMcpConnectors,
    retry: false,
    enabled: Boolean(session && !mustSignIn),
  })

  const [name, setName] = useState("")
  const [audience, setAudience] = useState("")
  const [scopePreset, setScopePreset] = useState("read")
  const [customSecret, setCustomSecret] = useState("")
  const [showCustomSecret, setShowCustomSecret] = useState(false)
  const [creating, setCreating] = useState(false)

  const [revealedSecret, setRevealedSecret] = useState<string | null>(null)
  const [revealedName, setRevealedName] = useState("")
  const [copiedBanner, setCopiedBanner] = useState(false)
  const [copiedEndpoint, setCopiedEndpoint] = useState(false)
  const [copiedSnippet, setCopiedSnippet] = useState(false)
  const [fallbackSecret, setFallbackSecret] = useState<{ label: string; value: string } | null>(null)

  const [testingId, setTestingId] = useState<string | null>(null)
  const [testResults, setTestResults] = useState<Record<string, McpConnectorTestResponse>>({})
  const [copyingId, setCopyingId] = useState<string | null>(null)

  const [editTarget, setEditTarget] = useState<McpConnectorItem | null>(null)
  const [editName, setEditName] = useState("")
  const [editAudience, setEditAudience] = useState("")
  const [editScopePreset, setEditScopePreset] = useState("read")
  const [editCustomSecret, setEditCustomSecret] = useState("")
  const [savingEdit, setSavingEdit] = useState(false)

  const [rotateTarget, setRotateTarget] = useState<McpConnectorItem | null>(null)
  const [rotating, setRotating] = useState(false)

  const [revokeTarget, setRevokeTarget] = useState<McpConnectorItem | null>(null)
  const [togglingRevoke, setTogglingRevoke] = useState(false)

  const [deleteTarget, setDeleteTarget] = useState<McpConnectorItem | null>(null)
  const [deleting, setDeleting] = useState(false)

  const isUnauthorized =
    mustSignIn ||
    (connectorsQuery.error instanceof ApiHttpError && connectorsQuery.error.status === 401)

  if (isUnauthorized) {
    return (
      <main className="flex-1 space-y-6">
        <PageHeader
          icon={Plug}
          title="MCP Connectors"
          description="Inbound Streamable HTTP MCP exposure, audience-scoped credentials, and live health probes."
        />
        <AuthGateCard
          nextPath="/mcp-connectors"
          oidcEnabled={session?.oidc_enabled}
          mgmtKeyConfigured={session?.mgmt_key_configured}
          onAuthenticated={() => queryClient.invalidateQueries()}
        />
      </main>
    )
  }

  const data = connectorsQuery.data
  const connectors = data?.connectors ?? []
  const origin = typeof window !== "undefined" ? window.location.origin : "https://styles.lengoc.me"
  const fullEndpoint = data?.endpoint
    ? data.endpoint.startsWith("http")
      ? data.endpoint
      : `${origin}${data.endpoint}`
    : `${origin}/mcp`

  const mcpSnippet = JSON.stringify(
    {
      mcpServers: {
        "lnc-styles": {
          type: "streamable-http",
          url: fullEndpoint,
          headers: {
            Authorization: "Bearer <USE_COPY_FROM_CONNECTOR_OR_API_KEY>",
          },
        },
      },
    },
    null,
    2
  )

  const handleCopyValue = async (secretValue: string, label: string) => {
    const ok = await copyText(secretValue, (manualValue) => {
      setFallbackSecret({ label, value: manualValue })
    })
    if (ok) {
      toast.success(`Copied "${label}" to clipboard`)
    } else {
      toast.info("Clipboard blocked — copy manually from dialog")
    }
    return ok
  }

  const handleCreate = async (e?: React.FormEvent) => {
    e?.preventDefault()
    const cleanName = name.trim()
    if (!cleanName) {
      toast.error("Enter a connector name.")
      return
    }
    const preset =
      CONNECTOR_SCOPE_PRESETS.find((p) => p.value === scopePreset) ?? CONNECTOR_SCOPE_PRESETS[0]
    setCreating(true)
    try {
      const res = await createMcpConnector({
        name: cleanName,
        ...(audience.trim() ? { audience: audience.trim() } : {}),
        scopes: preset.scopes,
        ...(customSecret.trim() ? { custom_secret: customSecret.trim() } : {}),
      })
      setName("")
      setAudience("")
      setCustomSecret("")
      setShowCustomSecret(false)
      setRevealedSecret(res.secret)
      setRevealedName(res.item.name)
      await queryClient.invalidateQueries({ queryKey: ["mcp-connectors"] })
      toast.success(`MCP connector "${res.item.name}" created`)
    } catch (err) {
      toast.error(err instanceof Error ? err.message : "Failed to create MCP connector")
    } finally {
      setCreating(false)
    }
  }

  const handleTest = async (connector: McpConnectorItem) => {
    setTestingId(connector.id)
    try {
      const res = await testMcpConnector(connector.id)
      setTestResults((prev) => ({ ...prev, [connector.id]: res }))
      if (res.ok) {
        toast.success(
          `Connector "${connector.name}" verified (${res.tools_count} tools, ${res.latency_ms}ms)`
        )
      } else {
        toast.warning(`Connector "${connector.name}": ${res.detail}`)
      }
    } catch (err) {
      toast.error(err instanceof Error ? err.message : "Connector test failed")
    } finally {
      setTestingId(null)
    }
  }

  const handleCopySecret = async (connector: McpConnectorItem) => {
    if (!connector.copy_available) {
      toast.info("No vault secret available. Rotate to seal a new secret in vault.")
      return
    }
    setCopyingId(connector.id)
    try {
      const res = await copyMcpConnector(connector.id)
      await handleCopyValue(res.secret, connector.name)
      await queryClient.invalidateQueries({ queryKey: ["admin-settings"] })
    } catch (err) {
      toast.error(err instanceof Error ? err.message : "Failed to copy connector secret")
    } finally {
      setCopyingId(null)
    }
  }

  const handleSaveEdit = async (e?: React.FormEvent) => {
    e?.preventDefault()
    if (!editTarget) return
    const cleanName = editName.trim()
    if (!cleanName) {
      toast.error("Connector name cannot be empty.")
      return
    }
    const preset =
      CONNECTOR_SCOPE_PRESETS.find((p) => p.value === editScopePreset) ?? CONNECTOR_SCOPE_PRESETS[0]
    setSavingEdit(true)
    try {
      const payload: {
        name?: string
        audience?: string
        scopes?: string[]
        custom_secret?: string
      } = {
        name: cleanName,
        ...(editAudience.trim() ? { audience: editAudience.trim() } : {}),
        scopes: preset.scopes,
      }
      // omitBlankSecrets: preserve existing HKDF-sealed secret when input is blank
      if (editCustomSecret.trim()) {
        payload.custom_secret = editCustomSecret.trim()
      }
      await updateMcpConnector(editTarget.id, payload)
      setEditTarget(null)
      setEditCustomSecret("")
      await queryClient.invalidateQueries({ queryKey: ["mcp-connectors"] })
      toast.success(`Updated connector "${cleanName}"`)
    } catch (err) {
      toast.error(err instanceof Error ? err.message : "Failed to update connector")
    } finally {
      setSavingEdit(false)
    }
  }

  const handleConfirmRotate = async () => {
    if (!rotateTarget) return
    setRotating(true)
    try {
      const res = await rotateMcpConnector(rotateTarget.id)
      setRevealedSecret(res.secret)
      setRevealedName(res.item.name)
      setRotateTarget(null)
      await queryClient.invalidateQueries({ queryKey: ["mcp-connectors"] })
      toast.success(`Rotated connector "${res.item.name}"`)
    } catch (err) {
      toast.error(err instanceof Error ? err.message : "Failed to rotate connector")
    } finally {
      setRotating(false)
    }
  }

  const handleToggleRevoke = async (connector: McpConnectorItem, revoke: boolean) => {
    setTogglingRevoke(true)
    try {
      await setMcpConnectorRevoked(connector.id, revoke)
      setRevokeTarget(null)
      await queryClient.invalidateQueries({ queryKey: ["mcp-connectors"] })
      toast.success(
        revoke
          ? `Revoked MCP connector "${connector.name}"`
          : `Re-enabled MCP connector "${connector.name}"`
      )
    } catch (err) {
      toast.error(err instanceof Error ? err.message : "Failed to update connector status")
    } finally {
      setTogglingRevoke(false)
    }
  }

  const handleConfirmDelete = async () => {
    if (!deleteTarget) return
    setDeleting(true)
    try {
      await deleteMcpConnector(deleteTarget.id)
      const deletedName = deleteTarget.name
      setDeleteTarget(null)
      await queryClient.invalidateQueries({ queryKey: ["mcp-connectors"] })
      toast.success(`Removed MCP connector "${deletedName}"`)
    } catch (err) {
      toast.error(err instanceof Error ? err.message : "Failed to remove connector")
    } finally {
      setDeleting(false)
    }
  }

  return (
    <main className="flex-1 space-y-6">
      <PageHeader
        icon={Plug}
        title="MCP Connectors"
        description="Inbound Streamable HTTP MCP exposure (/mcp), audience-scoped credentials, and live server health."
        actions={
          <>
            <Button asChild variant="outline" size="sm" className="h-7 whitespace-nowrap">
              <Link to="/keys">
                <KeyRound className="size-3.5" />
                <span>API Keys</span>
                <ArrowRight className="size-3.5" />
              </Link>
            </Button>
            <Button
              variant="outline"
              size="sm"
              className="h-7 whitespace-nowrap"
              onClick={async () => {
                const ok = await handleCopyValue(fullEndpoint, "MCP endpoint URL")
                if (ok) {
                  setCopiedEndpoint(true)
                  setTimeout(() => setCopiedEndpoint(false), 1800)
                }
              }}
            >
              {copiedEndpoint ? <Check className="size-3.5" /> : <Copy className="size-3.5" />}
              <span>{copiedEndpoint ? "Copied URL" : "Copy /mcp URL"}</span>
            </Button>
            <Button
              variant="outline"
              size="sm"
              className="h-7 whitespace-nowrap"
              onClick={() => connectorsQuery.refetch()}
              disabled={connectorsQuery.isFetching}
            >
              <RefreshCw
                className={cn("size-3.5", connectorsQuery.isFetching && "animate-spin")}
              />
              <span>Refresh</span>
            </Button>
          </>
        }
      />

      {/* Inbound MCP Exposure Posture Strip */}
      <Card className="border-border/70">
        <CardContent className="grid gap-3 py-4 sm:grid-cols-2 lg:grid-cols-4">
          <div className="space-y-1">
            <p className="text-[11px] font-medium tracking-wider text-muted-foreground uppercase">
              Inbound Endpoint
            </p>
            <code className="block truncate font-mono text-xs text-foreground">{fullEndpoint}</code>
          </div>
          <div className="space-y-1">
            <p className="text-[11px] font-medium tracking-wider text-muted-foreground uppercase">
              Transport & Tools
            </p>
            <div className="flex flex-wrap items-center gap-1.5">
              <Badge variant="secondary" className="text-[11px]">
                {data?.transport ?? "streamable-http"}
              </Badge>
              <Badge variant="outline" className="font-mono text-[11px]">
                {data?.tools_count ?? 8} tools ({data?.namespace ?? "styles__"})
              </Badge>
            </div>
          </div>
          <div className="space-y-1">
            <p className="text-[11px] font-medium tracking-wider text-muted-foreground uppercase">
              Auth & Audience
            </p>
            <div className="flex flex-wrap items-center gap-1.5">
              <Badge
                variant="outline"
                className={cn(
                  "text-[11px]",
                  data?.auth_required
                    ? "border-emerald-500/40 bg-emerald-500/10 text-emerald-500"
                    : "border-amber-500/40 bg-amber-500/10 text-amber-500"
                )}
              >
                {data?.auth_required ? "Bearer Auth Enforced" : "Open Read Mode"}
              </Badge>
              <code className="rounded bg-muted px-1.5 py-0.5 font-mono text-[11px]">
                {data?.audience ?? "lnc-styles-gallery"}
              </code>
            </div>
          </div>
          <div className="space-y-1">
            <p className="text-[11px] font-medium tracking-wider text-muted-foreground uppercase">
              Delegation Guard
            </p>
            <div className="flex flex-wrap items-center gap-1.5">
              <Badge variant="outline" className="text-[11px]">
                {data?.delegation_configured ? "Audience Verified" : "Default Audience"}
              </Badge>
              <span className="text-xs text-muted-foreground">
                Max: {(data?.max_scopes ?? ["mcp:read", "admin:reindex"]).join(", ")}
              </span>
            </div>
          </div>
        </CardContent>
      </Card>

      {revealedSecret ? (
        <Card className="border-emerald-500/40 bg-emerald-500/5">
          <CardHeader className="pb-2">
            <div className="flex flex-wrap items-center justify-between gap-2">
              <div className="flex items-center gap-2 text-emerald-500">
                <ShieldCheck className="size-4" />
                <CardTitle className="text-sm font-semibold">
                  Connector Credential Ready: {revealedName}
                </CardTitle>
              </div>
              <Button
                variant="ghost"
                size="sm"
                className="h-7 text-xs"
                onClick={() => {
                  setRevealedSecret(null)
                  setRevealedName("")
                }}
              >
                Dismiss
              </Button>
            </div>
            <CardDescription className="text-xs">
              Sealed in HKDF vault storage. Copy now or use the audited Copy action in the connector row.
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
                const ok = await handleCopyValue(revealedSecret, revealedName)
                if (ok) {
                  setCopiedBanner(true)
                  setTimeout(() => setCopiedBanner(false), 1800)
                }
              }}
            >
              {copiedBanner ? <Check className="size-3.5" /> : <Copy className="size-3.5" />}
              <span>{copiedBanner ? "Copied" : "Copy Secret"}</span>
            </Button>
          </CardContent>
        </Card>
      ) : null}

      {/* Create Inbound MCP Connector */}
      <Card className="border-border/70">
        <CardHeader className="pb-3">
          <CardTitle className="text-base">Add Inbound MCP Connector</CardTitle>
          <CardDescription className="text-xs">
            Provision an audience-bound credential for Claude Code, Cursor, Codex, or external agents connecting to /mcp.
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
            <div className="grid gap-3 md:grid-cols-[minmax(0,1fr)_minmax(0,220px)_minmax(0,260px)_auto]">
              <div className="space-y-1.5">
                <Label htmlFor="connector-name" className="text-xs">
                  Connector Name
                </Label>
                <Input
                  id="connector-name"
                  value={name}
                  onChange={(e) => setName(e.target.value)}
                  placeholder="e.g. cursor-desktop"
                  maxLength={80}
                />
              </div>

              <div className="space-y-1.5">
                <Label htmlFor="connector-audience" className="text-xs">
                  Audience (optional)
                </Label>
                <Input
                  id="connector-audience"
                  value={audience}
                  onChange={(e) => setAudience(e.target.value)}
                  placeholder={data?.audience ?? "lnc-styles-gallery"}
                  maxLength={120}
                />
              </div>

              <div className="space-y-1.5">
                <Label className="text-xs">Scope Preset</Label>
                <Select value={scopePreset} onValueChange={setScopePreset}>
                  <SelectTrigger className="w-full" aria-label="Connector scope preset">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    {CONNECTOR_SCOPE_PRESETS.map((preset) => (
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
                  <span>{creating ? "Creating..." : "Add Connector"}</span>
                </Button>
              </div>
            </div>

            <div className="space-y-2">
              <button
                type="button"
                className="text-xs font-medium text-muted-foreground underline-offset-4 hover:text-foreground hover:underline"
                onClick={() => setShowCustomSecret((prev) => !prev)}
              >
                {showCustomSecret ? "Hide optional custom secret" : "Use custom secret token (optional)"}
              </button>
              {showCustomSecret ? (
                <Input
                  type="password"
                  autoComplete="new-password"
                  value={customSecret}
                  onChange={(e) => setCustomSecret(e.target.value)}
                  placeholder="Leave blank to auto-generate lnc_mcp_... credential"
                />
              ) : null}
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Connector Table */}
      <Card className="border-border/70">
        <CardHeader className="pb-3">
          <CardTitle className="text-base">Configured Inbound MCP Connectors</CardTitle>
          <CardDescription className="text-xs">
            Environment-seeded connectors cannot be edited or deleted from the UI. Vault connectors support Test · Copy · Edit · Rotate · Revoke · Remove.
          </CardDescription>
        </CardHeader>
        <CardContent>
          {connectorsQuery.isLoading ? (
            <p className="py-6 text-center text-xs text-muted-foreground">
              Loading MCP connectors...
            </p>
          ) : connectors.length === 0 ? (
            <div className="rounded-xl border border-dashed border-border/70 py-8 text-center">
              <p className="text-sm font-medium">No MCP connectors configured yet</p>
              <p className="mt-1 text-xs text-muted-foreground">
                Add a connector above to issue an audience-scoped credential for /mcp.
              </p>
            </div>
          ) : (
            <div className="divide-y divide-border/60 rounded-xl border border-border/70">
              {connectors.map((connector) => {
                const isEnv = connector.source === "env"
                const probe = testResults[connector.id]
                return (
                  <div
                    key={connector.id}
                    data-connector-row={connector.id}
                    className="space-y-2 px-4 py-3.5"
                  >
                    <div className="flex flex-col gap-3 lg:flex-row lg:items-center lg:justify-between">
                      <div className="min-w-0 space-y-1.5">
                        <div className="flex flex-wrap items-center gap-2">
                          <span className="truncate font-medium text-sm">{connector.name}</span>
                          <Badge
                            variant="outline"
                            className={cn(
                              "text-[11px]",
                              connector.enabled
                                ? "border-emerald-500/40 bg-emerald-500/10 text-emerald-500"
                                : "border-destructive/40 bg-destructive/10 text-destructive"
                            )}
                          >
                            {connector.enabled ? "Active" : "Revoked"}
                          </Badge>
                          <Badge variant="secondary" className="gap-1 text-[11px]">
                            {isEnv ? <Lock className="size-3" /> : null}
                            <span>{isEnv ? "Env Managed" : "Vault Sealed"}</span>
                          </Badge>
                          <code className="rounded bg-muted px-1.5 py-0.5 font-mono text-[11px] text-muted-foreground">
                            {connector.masked}
                          </code>
                        </div>

                        <div className="flex flex-wrap items-center gap-x-3 gap-y-1 text-[11px] text-muted-foreground">
                          <span>Audience: {connector.audience}</span>
                          <span>Created: {formatDate(connector.created)}</span>
                          {connector.rotated_at ? (
                            <span>Rotated: {formatDate(connector.rotated_at)}</span>
                          ) : null}
                          {connector.revoked_at ? (
                            <span>Revoked: {formatDate(connector.revoked_at)}</span>
                          ) : null}
                          <span className="inline-flex flex-wrap items-center gap-1">
                            {connector.scopes.map((scope) => (
                              <Badge
                                key={scope}
                                variant="outline"
                                className="px-1.5 py-0 font-mono text-[10px]"
                              >
                                {scope}
                              </Badge>
                            ))}
                          </span>
                        </div>
                      </div>

                      <div
                        data-action-bar="mcp-connector"
                        className="flex flex-wrap items-center gap-1.5 sm:flex-nowrap shrink-0"
                      >
                        <Button
                          variant="outline"
                          size="sm"
                          className="h-7 shrink-0 px-2.5 text-xs whitespace-nowrap"
                          disabled={testingId === connector.id}
                          onClick={() => handleTest(connector)}
                        >
                          <Activity className="size-3.5" />
                          <span>{testingId === connector.id ? "Testing..." : "Test"}</span>
                        </Button>

                        <Button
                          variant="outline"
                          size="sm"
                          className="h-7 shrink-0 px-2.5 text-xs whitespace-nowrap"
                          disabled={!connector.copy_available || copyingId === connector.id}
                          title={
                            connector.copy_available
                              ? "Decrypt and copy connector credential (audited)"
                              : "Managed externally or legacy — rotate to seal vault copy"
                          }
                          onClick={() => handleCopySecret(connector)}
                        >
                          <Copy className="size-3.5" />
                          <span>{copyingId === connector.id ? "Copying..." : "Copy"}</span>
                        </Button>

                        <Button
                          variant="outline"
                          size="sm"
                          className="h-7 shrink-0 px-2.5 text-xs whitespace-nowrap"
                          disabled={isEnv}
                          title={isEnv ? "Env-managed connector cannot be edited in UI" : "Edit connector"}
                          onClick={() => {
                            setEditTarget(connector)
                            setEditName(connector.name)
                            setEditAudience(connector.audience)
                            setEditScopePreset(presetValueFromScopes(connector.scopes))
                            setEditCustomSecret("")
                          }}
                        >
                          <Pencil className="size-3.5" />
                          <span>Edit</span>
                        </Button>

                        <Button
                          variant="outline"
                          size="sm"
                          className="h-7 shrink-0 px-2.5 text-xs whitespace-nowrap"
                          disabled={isEnv}
                          title={isEnv ? "Env-managed connector cannot be rotated in UI" : "Rotate credential"}
                          onClick={() => setRotateTarget(connector)}
                        >
                          <RotateCw className="size-3.5" />
                          <span>Rotate</span>
                        </Button>

                        {connector.enabled ? (
                          <Button
                            variant="outline"
                            size="sm"
                            className="h-7 shrink-0 px-2.5 text-xs whitespace-nowrap text-amber-600 hover:text-amber-500 dark:text-amber-400"
                            disabled={isEnv}
                            title={isEnv ? "Env-managed connector cannot be revoked in UI" : "Revoke connector"}
                            onClick={() => setRevokeTarget(connector)}
                          >
                            <ShieldOff className="size-3.5" />
                            <span>Revoke</span>
                          </Button>
                        ) : (
                          <Button
                            variant="outline"
                            size="sm"
                            className="h-7 shrink-0 px-2.5 text-xs whitespace-nowrap text-emerald-600 hover:text-emerald-500 dark:text-emerald-400"
                            disabled={isEnv || togglingRevoke}
                            onClick={() => handleToggleRevoke(connector, false)}
                          >
                            <ShieldCheck className="size-3.5" />
                            <span>Enable</span>
                          </Button>
                        )}

                        <Button
                          variant="outline"
                          size="sm"
                          className="h-7 shrink-0 px-2.5 text-xs whitespace-nowrap text-destructive hover:bg-destructive/10 hover:text-destructive"
                          disabled={isEnv}
                          title={isEnv ? "Env-managed connector cannot be deleted in UI" : "Remove connector"}
                          onClick={() => setDeleteTarget(connector)}
                        >
                          <Trash2 className="size-3.5" />
                          <span>Remove</span>
                        </Button>
                      </div>
                    </div>

                    {probe ? (
                      <div
                        className={cn(
                          "flex flex-wrap items-center justify-between gap-2 rounded-lg border px-3 py-1.5 text-xs",
                          probe.ok
                            ? "border-emerald-500/30 bg-emerald-500/5 text-emerald-600 dark:text-emerald-400"
                            : "border-amber-500/30 bg-amber-500/5 text-amber-600 dark:text-amber-400"
                        )}
                      >
                        <span>
                          Live probe ({probe.status}): {probe.detail}
                        </span>
                        <span className="font-mono text-[11px]">
                          {probe.tools_count} tools · {probe.latency_ms}ms
                        </span>
                      </div>
                    ) : null}
                  </div>
                )
              })}
            </div>
          )}
        </CardContent>
      </Card>

      {/* Exposed MCP Tools & Quick Client Snippet */}
      <div className="grid gap-6 lg:grid-cols-2">
        <Card className="border-border/70">
          <CardHeader className="pb-3">
            <div className="flex items-center gap-2">
              <Wrench className="size-4 text-primary" />
              <CardTitle className="text-base">
                Exposed MCP Tools ({data?.tools_count ?? data?.tools.length ?? 8})
              </CardTitle>
            </div>
            <CardDescription className="text-xs">
              Read-only design intelligence tools advertised on /mcp over Streamable HTTP.
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-2">
            {(data?.tools ?? []).map((tool) => (
              <div
                key={tool.name}
                className="flex flex-col gap-0.5 rounded-lg border border-border/60 bg-muted/20 px-3 py-2"
              >
                <code className="font-mono text-xs font-semibold text-foreground">{tool.name}</code>
                <p className="text-xs text-muted-foreground">{tool.description}</p>
              </div>
            ))}
          </CardContent>
        </Card>

        <Card className="border-border/70">
          <CardHeader className="pb-3">
            <div className="flex flex-wrap items-center justify-between gap-2">
              <div>
                <CardTitle className="text-base">Client Config Snippet (.mcp.json)</CardTitle>
                <CardDescription className="text-xs">
                  Paste into Claude Code, Cursor, or Codex and insert a copied connector token.
                </CardDescription>
              </div>
              <Button
                variant="outline"
                size="sm"
                className="h-7 shrink-0 whitespace-nowrap"
                onClick={async () => {
                  const ok = await handleCopyValue(mcpSnippet, ".mcp.json snippet")
                  if (ok) {
                    setCopiedSnippet(true)
                    setTimeout(() => setCopiedSnippet(false), 1800)
                  }
                }}
              >
                {copiedSnippet ? <Check className="size-3.5" /> : <Copy className="size-3.5" />}
                <span>{copiedSnippet ? "Copied" : "Copy Snippet"}</span>
              </Button>
            </div>
          </CardHeader>
          <CardContent>
            <pre className="overflow-x-auto rounded-xl border border-border/70 bg-muted/30 p-3.5 font-mono text-xs leading-relaxed text-foreground">
              {mcpSnippet}
            </pre>
          </CardContent>
        </Card>
      </div>

      {/* Edit Connector Dialog */}
      <Dialog open={Boolean(editTarget)} onOpenChange={(open) => !open && setEditTarget(null)}>
        <DialogContent>
          <div
            className="space-y-4"
            onKeyDown={(e) => {
              if (e.key === "Enter" && e.target instanceof HTMLInputElement && !savingEdit) {
                e.preventDefault()
                void handleSaveEdit()
              }
            }}
          >
            <DialogHeader>
              <DialogTitle>Edit MCP Connector</DialogTitle>
              <DialogDescription>
                Update metadata or supply a replacement credential. Leave the secret field blank to keep the existing HKDF-sealed token.
              </DialogDescription>
            </DialogHeader>
            <div className="space-y-3">
              <div className="space-y-1.5">
                <Label htmlFor="edit-connector-name" className="text-xs">
                  Connector Name
                </Label>
                <Input
                  id="edit-connector-name"
                  value={editName}
                  onChange={(e) => setEditName(e.target.value)}
                  maxLength={80}
                />
              </div>
              <div className="space-y-1.5">
                <Label htmlFor="edit-connector-audience" className="text-xs">
                  Audience
                </Label>
                <Input
                  id="edit-connector-audience"
                  value={editAudience}
                  onChange={(e) => setEditAudience(e.target.value)}
                  maxLength={120}
                />
              </div>
              <div className="space-y-1.5">
                <Label className="text-xs">Scopes</Label>
                <Select value={editScopePreset} onValueChange={setEditScopePreset}>
                  <SelectTrigger className="w-full" aria-label="Edit connector scopes">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    {CONNECTOR_SCOPE_PRESETS.map((preset) => (
                      <SelectItem key={preset.value} value={preset.value}>
                        {preset.label}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
              <div className="space-y-1.5">
                <Label htmlFor="edit-connector-secret" className="text-xs">
                  Replace Secret (optional)
                </Label>
                <Input
                  id="edit-connector-secret"
                  type="password"
                  autoComplete="new-password"
                  value={editCustomSecret}
                  onChange={(e) => setEditCustomSecret(e.target.value)}
                  placeholder="******** (leave blank to keep existing secret)"
                />
              </div>
            </div>
            <DialogFooter>
              <Button type="button" variant="outline" onClick={() => setEditTarget(null)}>
                Cancel
              </Button>
              <Button type="button" disabled={savingEdit} onClick={() => void handleSaveEdit()}>
                {savingEdit ? "Saving..." : "Save Connector"}
              </Button>
            </DialogFooter>
          </div>
        </DialogContent>
      </Dialog>

      {/* Rotate Connector Dialog */}
      <Dialog open={Boolean(rotateTarget)} onOpenChange={(open) => !open && setRotateTarget(null)}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Rotate MCP Connector?</DialogTitle>
            <DialogDescription>
              Rotating <strong>{rotateTarget?.name}</strong> generates a fresh credential sealed in vault storage and immediately invalidates the old token on connected MCP clients.
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

      {/* Revoke Connector Dialog */}
      <Dialog open={Boolean(revokeTarget)} onOpenChange={(open) => !open && setRevokeTarget(null)}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle className="flex items-center gap-2">
              <AlertTriangle className="size-4 text-amber-500" />
              <span>Revoke MCP Connector?</span>
            </DialogTitle>
            <DialogDescription>
              All MCP clients authenticating as <strong>{revokeTarget?.name}</strong> will be rejected immediately on <code>/mcp</code> until re-enabled.
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
              {togglingRevoke ? "Revoking..." : "Revoke Connector"}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Remove Connector Dialog */}
      <Dialog open={Boolean(deleteTarget)} onOpenChange={(open) => !open && setDeleteTarget(null)}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle className="flex items-center gap-2 text-destructive">
              <Trash2 className="size-4" />
              <span>Remove MCP Connector?</span>
            </DialogTitle>
            <DialogDescription>
              Permanently removes <strong>{deleteTarget?.name}</strong> and purges its encrypted secret envelope. Connected MCP tools using this credential will stop immediately.
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

      {/* Manual HTTP Clipboard Fallback */}
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
              Automatic clipboard access was blocked for <strong>{fallbackSecret?.label}</strong>. Copy manually below, then close.
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
