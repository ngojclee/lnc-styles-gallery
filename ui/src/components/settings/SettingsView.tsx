import { useEffect, useState } from "react"
import { useQuery, useQueryClient } from "@tanstack/react-query"
import {
  Blocks,
  CheckCircle2,
  Database,
  FileCode2,
  FolderKanban,
  HardDrive,
  Image as ImageIcon,
  KeyRound,
  Lock,
  Palette,
  RefreshCw,
  Server,
  Settings2,
  ShieldAlert,
  ShieldCheck,
  XCircle,
} from "lucide-react"
import { toast } from "sonner"

import {
  ApiHttpError,
  authSessionQuery,
  clearAuthSessionCache,
  fetchAdminSettings,
  fetchAdminStatus,
  revokeAllSessions,
  triggerReindex,
  updateAdminSettings,
} from "@/api/styles"
import { AuthGateCard } from "@/components/common/AuthGateCard"
import { PageHeader } from "@/components/common/PageHeader"
import { StatCard } from "@/components/common/StatCard"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Input } from "@/components/ui/input"
import { cn } from "@/lib/utils"

function formatDuration(secs?: number): string {
  if (!secs || secs <= 0) return "Default"
  if (secs % 86400 === 0) return `${secs / 86400}d`
  if (secs % 3600 === 0) return `${secs / 3600}h`
  if (secs % 60 === 0) return `${secs / 60}m`
  return `${secs}s`
}

export function SettingsView() {
  const queryClient = useQueryClient()
  const sessionQuery = useQuery(authSessionQuery)
  const session = sessionQuery.data

  const [reindexing, setReindexing] = useState(false)
  const [revokingSessions, setRevokingSessions] = useState(false)
  const [savingUrl, setSavingUrl] = useState(false)
  const [publicUrlInput, setPublicUrlInput] = useState("")

  const mustSignIn = Boolean(session?.auth_required && !session?.authenticated)

  const statusQuery = useQuery({
    queryKey: ["admin-status", session?.authenticated, session?.session_epoch],
    queryFn: fetchAdminStatus,
    enabled: Boolean(session && !mustSignIn),
    retry: false,
  })

  const settingsQuery = useQuery({
    queryKey: ["admin-settings", session?.authenticated, session?.session_epoch],
    queryFn: fetchAdminSettings,
    enabled: Boolean(session && !mustSignIn),
    retry: false,
  })

  const settings = settingsQuery.data

  useEffect(() => {
    if (settings?.public_base_url) {
      setPublicUrlInput(settings.public_base_url)
    }
  }, [settings?.public_base_url])

  const isUnauthorized =
    mustSignIn ||
    (statusQuery.error instanceof ApiHttpError && statusQuery.error.status === 401) ||
    (settingsQuery.error instanceof ApiHttpError && settingsQuery.error.status === 401)

  if (isUnauthorized) {
    return (
      <main className="flex-1 space-y-6">
        <PageHeader
          icon={Settings2}
          title="Settings"
          description="Authentication posture, runtime configuration, and read-only corpus diagnostics."
        />
        <AuthGateCard
          nextPath="/settings"
          oidcEnabled={session?.oidc_enabled}
          mgmtKeyConfigured={session?.mgmt_key_configured}
          onAuthenticated={() => queryClient.invalidateQueries()}
        />
      </main>
    )
  }

  const handleReindex = async () => {
    setReindexing(true)
    try {
      const res = await triggerReindex()
      await queryClient.invalidateQueries()
      toast.success(
        `Re-indexed ${res.summary.styles_count} styles, ${res.summary.templates_count} templates, ${res.summary.components_count ?? 0} components`
      )
    } catch (err) {
      toast.error(err instanceof Error ? err.message : "Re-index failed")
    } finally {
      setReindexing(false)
    }
  }

  const handleRevokeAll = async () => {
    setRevokingSessions(true)
    try {
      const res = await revokeAllSessions()
      clearAuthSessionCache(queryClient)
      toast.success(`All browser sessions revoked (epoch v${res.session_epoch})`)
      await queryClient.invalidateQueries({ queryKey: authSessionQuery.queryKey })
    } catch (err) {
      toast.error(err instanceof Error ? err.message : "Failed to revoke sessions")
    } finally {
      setRevokingSessions(false)
    }
  }

  const handleSavePublicUrl = async (e: React.FormEvent) => {
    e.preventDefault()
    const trimmed = publicUrlInput.trim()
    if (!trimmed) {
      toast.error("Public base URL cannot be empty")
      return
    }
    setSavingUrl(true)
    try {
      await updateAdminSettings({ public_base_url: trimmed })
      await queryClient.invalidateQueries({ queryKey: ["admin-settings"] })
      await queryClient.invalidateQueries({ queryKey: ["mcp-connectors"] })
      toast.success("Public base URL updated")
    } catch (err) {
      toast.error(err instanceof Error ? err.message : "Failed to update URL")
    } finally {
      setSavingUrl(false)
    }
  }

  const status = statusQuery.data

  const dataMounts = [
    {
      label: "Refero Design Styles",
      path: settings?.refero_dir ?? "data/refero-styles",
      exists: status?.refero_dir_exists ?? settings?.refero_dir_exists ?? false,
      count: status?.styles_count ?? 0,
      unit: "styles",
    },
    {
      label: "Prompt-Driven HTML Templates",
      path: settings?.prompt_ui_dir ?? "data/prompt-driven-ui",
      exists: status?.prompt_ui_dir_exists ?? settings?.prompt_ui_dir_exists ?? false,
      count: status?.templates_count ?? 0,
      unit: "templates",
    },
    {
      label: "LNC Components & Effects",
      path: settings?.components_dir ?? "data/lnc-components",
      exists: status?.components_dir_exists ?? settings?.components_dir_exists ?? false,
      count: status?.components_count ?? 0,
      unit: "items",
    },
  ]

  const oidcUsers = settings?.oidc_allowed_users ?? []
  const oidcGroups = settings?.oidc_allowed_groups ?? []

  return (
    <main className="flex-1 space-y-6">
      <PageHeader
        icon={Settings2}
        title="Settings"
        description="Read-only Authentication Posture, corpus re-indexing, and runtime endpoint configuration."
        actions={
          <div className="flex flex-wrap items-center gap-2">
            {session?.auth_required ? (
              <Button
                variant="outline"
                size="sm"
                className="h-8 gap-1.5 text-destructive hover:bg-destructive/10 hover:text-destructive"
                onClick={handleRevokeAll}
                disabled={revokingSessions}
              >
                <ShieldAlert className="size-3.5" />
                {revokingSessions ? "Revoking..." : "Revoke All Sessions"}
              </Button>
            ) : null}
            <Button onClick={handleReindex} disabled={reindexing} size="sm" className="h-8">
              <RefreshCw className={cn("mr-1.5 size-3.5", reindexing && "animate-spin")} />
              {reindexing ? "Re-indexing..." : "Hot Re-index Corpus"}
            </Button>
          </div>
        }
      />

      {/* Corpus Stats */}
      <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-5">
        <StatCard
          label="Indexed Styles"
          value={status?.styles_count ?? "—"}
          hint={`${status?.designs_count ?? 0} with DESIGN.md`}
          icon={Palette}
        />
        <StatCard
          label="Web Previews"
          value={status?.previews_count ?? "—"}
          hint="Screenshots + rendered SVGs"
          icon={ImageIcon}
        />
        <StatCard
          label="Collections"
          value={status?.collections_count ?? "—"}
          hint="Editorial & curated groups"
          icon={FolderKanban}
        />
        <StatCard
          label="HTML Templates"
          value={status?.templates_count ?? "—"}
          hint="EN + VI sample designs"
          icon={FileCode2}
        />
        <StatCard
          label="Components"
          value={status?.components_count ?? 0}
          hint="LNC demos & UI blocks"
          icon={Blocks}
        />
      </div>

      {/* Authentication Posture (Strictly Read-Only — Zero Secret Inputs) */}
      <Card>
        <CardHeader>
          <div className="flex flex-wrap items-center justify-between gap-2">
            <div>
              <CardTitle className="flex items-center gap-2 text-base">
                <ShieldCheck className="size-4 text-primary" />
                Authentication Posture
              </CardTitle>
              <CardDescription>
                Read-only security overview. OIDC credentials and master encryption keys are injected via environment variables and never editable or exposed in the browser.
              </CardDescription>
            </div>
            <Badge
              variant="outline"
              className="gap-1 border-emerald-500/30 bg-emerald-500/10 text-[11px] font-medium text-emerald-500"
            >
              <Lock className="size-3" />
              Read-Only Posture
            </Badge>
          </div>
        </CardHeader>
        <CardContent>
          {settings ? (
            <div className="grid gap-6 lg:grid-cols-2">
              <dl className="divide-y divide-border/50 text-xs">
                <div className="flex items-center justify-between gap-3 py-2.5">
                  <dt className="text-muted-foreground">Effective Auth Mode</dt>
                  <dd>
                    <Badge variant="secondary" className="font-mono text-[11px]">
                      {String(settings.settings?.auth_mode ?? (settings.oidc_enabled ? "oidc+api_key" : "api_key"))}
                    </Badge>
                  </dd>
                </div>
                <div className="flex items-center justify-between gap-3 py-2.5">
                  <dt className="text-muted-foreground">OIDC SSO (PKCE S256)</dt>
                  <dd className="flex items-center gap-2 font-mono text-foreground">
                    <Badge
                      variant={settings.oidc_enabled ? "default" : "outline"}
                      className="text-[10px]"
                    >
                      {settings.oidc_enabled ? "Enabled" : "Disabled"}
                    </Badge>
                    {settings.oidc_client_secret_configured ? (
                      <span className="text-[11px] text-emerald-500">$enc:1 sealed</span>
                    ) : null}
                  </dd>
                </div>
                <div className="flex items-center justify-between gap-3 py-2.5">
                  <dt className="text-muted-foreground">OIDC Issuer</dt>
                  <dd
                    className="max-w-[280px] truncate font-mono text-foreground"
                    title={settings.oidc_issuer || "—"}
                  >
                    {settings.oidc_issuer || "Not configured"}
                  </dd>
                </div>
                <div className="flex items-center justify-between gap-3 py-2.5">
                  <dt className="text-muted-foreground">OIDC Client / Scopes</dt>
                  <dd className="max-w-[280px] truncate font-mono text-foreground">
                    {settings.oidc_client_id || "—"}
                    {settings.oidc_scopes ? ` (${settings.oidc_scopes})` : ""}
                  </dd>
                </div>
                <div className="flex items-center justify-between gap-3 py-2.5">
                  <dt className="text-muted-foreground">OIDC Redirect URI</dt>
                  <dd
                    className="max-w-[280px] truncate font-mono text-foreground"
                    title={settings.oidc_redirect_uri || "—"}
                  >
                    {settings.oidc_redirect_uri || "—"}
                  </dd>
                </div>
                <div className="flex items-start justify-between gap-3 py-2.5">
                  <dt className="shrink-0 text-muted-foreground">Allowed Users / Groups</dt>
                  <dd className="flex flex-wrap justify-end gap-1 font-mono">
                    {oidcUsers.length === 0 && oidcGroups.length === 0 ? (
                      <span className="text-amber-500">Fail-closed (empty allowlist)</span>
                    ) : (
                      <>
                        {oidcUsers.map((u) => (
                          <Badge key={`u-${u}`} variant="outline" className="text-[10px]">
                            user:{u}
                          </Badge>
                        ))}
                        {oidcGroups.map((g) => (
                          <Badge key={`g-${g}`} variant="outline" className="text-[10px]">
                            group:{g}
                          </Badge>
                        ))}
                      </>
                    )}
                  </dd>
                </div>
              </dl>

              <dl className="divide-y divide-border/50 text-xs">
                <div className="flex items-center justify-between gap-3 py-2.5">
                  <dt className="text-muted-foreground">Break-glass Management Key</dt>
                  <dd className="flex items-center gap-1.5 font-mono text-foreground">
                    <KeyRound className="size-3.5 text-muted-foreground" />
                    {settings.api_key_configured
                      ? `Argon2id + HMAC (${settings.api_key_masked})`
                      : "Not configured"}
                  </dd>
                </div>
                <div className="flex items-center justify-between gap-3 py-2.5">
                  <dt className="text-muted-foreground">Master Key (`LNC_MASTER_KEY`)</dt>
                  <dd className="flex flex-wrap items-center justify-end gap-1.5 font-mono text-foreground">
                    <Badge
                      variant={settings.master_key_configured ? "default" : "outline"}
                      className="text-[10px]"
                    >
                      {settings.master_key_configured ? "Env Pinned" : "Dev Fallback"}
                    </Badge>
                    {settings.master_key_old_configured ? (
                      <Badge variant="secondary" className="text-[10px]">
                        Rotation Key Present
                      </Badge>
                    ) : null}
                    {settings.rewrapped_from_old_key ? (
                      <Badge variant="secondary" className="text-[10px] text-emerald-500">
                        Vault Re-wrapped
                      </Badge>
                    ) : null}
                  </dd>
                </div>
                <div className="flex items-center justify-between gap-3 py-2.5">
                  <dt className="text-muted-foreground">Session Cookie (`v2` HttpOnly)</dt>
                  <dd className="font-mono text-foreground">
                    Idle {formatDuration(settings.session_idle_ttl_seconds)} · Max{" "}
                    {formatDuration(settings.session_max_ttl_seconds)} · Secure=
                    {settings.cookie_secure ? "on" : "auto"}
                  </dd>
                </div>
                <div className="flex items-center justify-between gap-3 py-2.5">
                  <dt className="text-muted-foreground">Session Epoch</dt>
                  <dd className="font-mono text-foreground">
                    v{settings.session_epoch ?? session?.session_epoch ?? 1}
                  </dd>
                </div>
                <div className="flex items-center justify-between gap-3 py-2.5">
                  <dt className="text-muted-foreground">MCP Bearer Auth Gate</dt>
                  <dd className="font-mono text-foreground">
                    {settings.mcp_auth_required ? "Required (Bearer / X-API-Key)" : "Open"} · Aud:{" "}
                    {settings.connector_audience ?? "styles-mcp"}
                  </dd>
                </div>
                <div className="flex items-center justify-between gap-3 py-2.5">
                  <dt className="text-muted-foreground">Delegation Secret</dt>
                  <dd className="font-mono text-foreground">
                    {settings.connector_delegation_configured
                      ? "Configured (Env)"
                      : "Not required (Bearer only)"}
                  </dd>
                </div>
              </dl>
            </div>
          ) : (
            <div className="py-8 text-center text-xs text-muted-foreground">
              Loading authentication posture...
            </div>
          )}
        </CardContent>
      </Card>

      <div className="grid gap-6 lg:grid-cols-2">
        {/* Data Mounts */}
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2 text-base">
              <HardDrive className="size-4 text-primary" />
              Read-Only Data Volume Mounts
            </CardTitle>
            <CardDescription>
              The service runs strictly read-only against corpus directories and maintains in-memory indexes plus config vault state in `STYLES_CONFIG_DIR`.
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-3">
            {dataMounts.map((m) => (
              <div
                key={m.label}
                className="flex items-center justify-between gap-3 rounded-lg border border-border/60 bg-muted/20 p-3"
              >
                <div className="min-w-0">
                  <div className="flex items-center gap-2">
                    {m.exists ? (
                      <CheckCircle2 className="size-4 shrink-0 text-emerald-500" />
                    ) : (
                      <XCircle className="size-4 shrink-0 text-destructive" />
                    )}
                    <span className="truncate text-sm font-medium">{m.label}</span>
                  </div>
                  <p className="mt-1 truncate font-mono text-[11px] text-muted-foreground">
                    {m.path}
                  </p>
                </div>
                <Badge variant="outline" className="shrink-0 font-mono text-xs">
                  {m.count} {m.unit}
                </Badge>
              </div>
            ))}
          </CardContent>
        </Card>

        {/* Runtime Info & Public URL */}
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2 text-base">
              <Server className="size-4 text-primary" />
              Runtime Environment & Public Base URL
            </CardTitle>
            <CardDescription>
              Active server metadata and public URL used for generated `/mcp` endpoint links.
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            {settings ? (
              <>
                <form onSubmit={handleSavePublicUrl} className="flex flex-col gap-2 sm:flex-row">
                  <Input
                    value={publicUrlInput}
                    onChange={(e) => setPublicUrlInput(e.target.value)}
                    placeholder="https://styles.lengoc.me"
                    className="h-8 font-mono text-xs"
                    aria-label="Public Base URL"
                  />
                  <Button type="submit" size="sm" className="h-8 shrink-0" disabled={savingUrl}>
                    {savingUrl ? "Saving..." : "Save Base URL"}
                  </Button>
                </form>

                <dl className="divide-y divide-border/50 text-xs">
                  <div className="flex justify-between gap-2 py-2">
                    <dt className="text-muted-foreground">Service / Runtime</dt>
                    <dd className="font-mono text-foreground">{settings.service_name} (Rust/Axum)</dd>
                  </div>
                  <div className="flex justify-between gap-2 py-2">
                    <dt className="text-muted-foreground">Version</dt>
                    <dd className="font-mono text-foreground">{settings.service_version}</dd>
                  </div>
                  <div className="flex justify-between gap-2 py-2">
                    <dt className="text-muted-foreground">Bind Host / Port</dt>
                    <dd className="font-mono text-foreground">
                      {settings.host}:{settings.port}
                    </dd>
                  </div>
                  <div className="flex justify-between gap-2 py-2">
                    <dt className="text-muted-foreground">Data Root (Read-Only)</dt>
                    <dd
                      className="max-w-[260px] truncate font-mono text-foreground"
                      title={settings.data_root}
                    >
                      {settings.data_root}
                    </dd>
                  </div>
                  <div className="flex justify-between gap-2 py-2">
                    <dt className="text-muted-foreground">Allowed Hosts</dt>
                    <dd className="font-mono text-foreground">
                      {settings.allowed_hosts ?? settings.cors_origins}
                    </dd>
                  </div>
                </dl>
              </>
            ) : (
              <div className="py-8 text-center text-xs text-muted-foreground">
                Loading runtime configuration...
              </div>
            )}
          </CardContent>
        </Card>
      </div>

      {/* Category Breakdown */}
      {status?.categories && status.categories.length > 0 ? (
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2 text-base">
              <Database className="size-4 text-primary" />
              Indexed Category Breakdown
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="grid gap-3 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4">
              {status.categories.map((cat) => (
                <div
                  key={cat.id}
                  className="flex items-center justify-between rounded-lg border border-border/60 bg-muted/20 px-3.5 py-2.5 text-xs"
                >
                  <span className="font-medium text-foreground">{cat.label}</span>
                  <Badge variant="secondary" className="font-mono">
                    {cat.count}
                  </Badge>
                </div>
              ))}
            </div>
          </CardContent>
        </Card>
      ) : null}

      {/* Recent Secret Copy Audit Log */}
      {settings?.recent_audit_events && settings.recent_audit_events.length > 0 ? (
        <Card>
          <CardHeader>
            <CardTitle className="text-base">Recent Audited Vault Copy Events</CardTitle>
            <CardDescription>
              Immutable append-only `audit.jsonl` records for `POST .../copy` operations (never records secrets).
            </CardDescription>
          </CardHeader>
          <CardContent>
            <div className="divide-y divide-border/50 text-xs">
              {settings.recent_audit_events.map((ev, idx) => (
                <div
                  key={`${ev.ts}-${ev.record_id}-${idx}`}
                  className="flex flex-wrap items-center justify-between gap-2 py-2 font-mono"
                >
                  <div className="flex items-center gap-2">
                    <Badge variant="outline" className="text-[10px]">
                      {ev.action}
                    </Badge>
                    <span className="text-foreground">
                      {ev.kind}:{ev.record_id}
                    </span>
                  </div>
                  <div className="flex items-center gap-3 text-[11px] text-muted-foreground">
                    <span>actor={ev.actor}</span>
                    <span>ip={ev.ip}</span>
                    <span>{new Date(ev.ts).toLocaleString()}</span>
                  </div>
                </div>
              ))}
            </div>
          </CardContent>
        </Card>
      ) : null}
    </main>
  )
}
