import { useState } from "react"
import { createFileRoute } from "@tanstack/react-router"
import { useQuery, useQueryClient } from "@tanstack/react-query"
import {
  Blocks,
  CheckCircle2,
  Database,
  FileCode2,
  FolderKanban,
  HardDrive,
  Image as ImageIcon,
  Palette,
  RefreshCw,
  Server,
  Settings2,
  ShieldAlert,
  XCircle,
} from "lucide-react"
import { toast } from "sonner"

import {
  ApiHttpError,
  authSessionQuery,
  fetchAdminSettings,
  fetchAdminStatus,
  revokeAllSessions,
  triggerReindex,
} from "@/api/styles"
import { AuthGateCard } from "@/components/common/AuthGateCard"
import { PageHeader } from "@/components/common/PageHeader"
import { StatCard } from "@/components/common/StatCard"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { cn } from "@/lib/utils"

export const Route = createFileRoute("/admin")({
  component: AdminPage,
})

function AdminPage() {
  const queryClient = useQueryClient()
  const sessionQuery = useQuery(authSessionQuery)
  const session = sessionQuery.data

  const [reindexing, setReindexing] = useState(false)
  const [revokingSessions, setRevokingSessions] = useState(false)

  const mustSignIn = Boolean(session?.auth_required && !session?.authenticated)

  const statusQuery = useQuery({
    queryKey: ["admin-status", session?.authenticated, session?.session_epoch],
    queryFn: fetchAdminStatus,
    enabled: !mustSignIn,
    retry: false,
  })

  const settingsQuery = useQuery({
    queryKey: ["admin-settings", session?.authenticated, session?.session_epoch],
    queryFn: fetchAdminSettings,
    enabled: !mustSignIn,
    retry: false,
  })

  const isUnauthorized =
    (statusQuery.error instanceof ApiHttpError && statusQuery.error.status === 401) ||
    (settingsQuery.error instanceof ApiHttpError && settingsQuery.error.status === 401)

  if (mustSignIn || isUnauthorized) {
    return (
      <main className="flex-1 space-y-6">
        <PageHeader
          icon={Settings2}
          title="Corpus Admin & Runtime Diagnostics"
          description="Sign in with LNC SSO or break-glass management key to inspect corpus diagnostics or trigger hot re-indexing."
        />
        <AuthGateCard
          nextPath="/admin"
          oidcEnabled={session?.oidc_enabled ?? true}
          mgmtKeyConfigured={session?.mgmt_key_configured ?? true}
          onAuthenticated={() => {
            queryClient.invalidateQueries()
          }}
        />
      </main>
    )
  }

  const status = statusQuery.data
  const settings = settingsQuery.data

  const handleReindex = async () => {
    setReindexing(true)
    try {
      const res = await triggerReindex()
      await queryClient.invalidateQueries()
      toast.success(
        `Re-indexed ${res.summary.styles_count} styles (${res.summary.previews_count ?? 0} previews), ${res.summary.templates_count} templates & ${res.summary.components_count ?? 0} components`
      )
    } catch (err) {
      toast.error(err instanceof Error ? err.message : "Re-index failed")
    } finally {
      setReindexing(false)
    }
  }

  const handleRevokeAll = async () => {
    if (
      !window.confirm(
        "Invalidate all active browser sessions across every device by incrementing session_epoch?"
      )
    ) {
      return
    }
    setRevokingSessions(true)
    try {
      const res = await revokeAllSessions()
      await queryClient.invalidateQueries()
      toast.success(`All sessions revoked (epoch ${res.session_epoch})`)
    } catch (err) {
      toast.error(err instanceof Error ? err.message : "Failed to revoke sessions")
    } finally {
      setRevokingSessions(false)
    }
  }

  const indexedTime = status?.indexed_at
    ? new Date(status.indexed_at * 1000).toLocaleString()
    : "—"

  return (
    <main className="flex-1 space-y-6">
      <PageHeader
        icon={Settings2}
        title="Corpus Admin & Runtime Diagnostics"
        description="Inspect read-only NAS/local corpus paths, trigger zero-downtime hot re-indexing, and manage session security."
        actions={
          <div className="flex flex-wrap items-center gap-2">
            {session?.authenticated ? (
              <Button
                variant="outline"
                onClick={handleRevokeAll}
                disabled={revokingSessions}
                className="gap-1.5 text-destructive hover:bg-destructive/10 hover:text-destructive"
              >
                <ShieldAlert className="size-4" />
                <span>{revokingSessions ? "Revoking..." : "Revoke All Sessions"}</span>
              </Button>
            ) : null}
            <Button onClick={handleReindex} disabled={reindexing}>
              <RefreshCw className={cn("size-4", reindexing && "animate-spin")} />
              {reindexing ? "Scanning Corpora..." : "Trigger Hot Re-Index"}
            </Button>
          </div>
        }
      />

      {/* Corpus Metrics */}
      <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-5">
        <StatCard
          label="Indexed Styles"
          value={status?.styles_count ?? "—"}
          hint={`${status?.designs_count ?? 0} DESIGN.md files loaded`}
          icon={Palette}
        />
        <StatCard
          label="Harvested Previews"
          value={status?.previews_count ?? 0}
          hint="styles/<uuid>/preview.* files on disk"
          icon={ImageIcon}
        />
        <StatCard
          label="Style Collections"
          value={status?.collections_count ?? "—"}
          hint="Category, theme & sitemap groups"
          icon={FolderKanban}
        />
        <StatCard
          label="HTML Templates"
          value={status?.templates_count ?? "—"}
          hint="EN + VI prompt-driven-ui"
          icon={FileCode2}
        />
        <StatCard
          label="UI Components"
          value={status?.components_count ?? 0}
          hint="LNC HTML + Knowns TSX"
          icon={Blocks}
        />
      </div>

      <div className="grid gap-6 lg:grid-cols-2">
        {/* Corpus Directory Health */}
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2 text-base">
              <HardDrive className="size-4 text-primary" />
              Corpus Storage Paths (Read-Only)
            </CardTitle>
            <CardDescription className="text-xs">
              Last indexed at: <span className="font-mono text-foreground">{indexedTime}</span>
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-3.5 text-xs">
            <div className="rounded-xl border border-border/70 bg-muted/25 p-3.5">
              <div className="mb-1.5 flex items-center justify-between">
                <span className="font-semibold text-foreground">Refero Styles Corpus</span>
                {status?.refero_dir_exists ? (
                  <Badge variant="outline" className="gap-1 border-emerald-500/40 text-emerald-500">
                    <CheckCircle2 className="size-3" /> Mounted
                  </Badge>
                ) : (
                  <Badge variant="destructive" className="gap-1">
                    <XCircle className="size-3" /> Missing
                  </Badge>
                )}
              </div>
              <code className="block break-all font-mono text-[11px] text-muted-foreground">
                {status?.refero_dir ?? "Loading..."}
              </code>
            </div>

            <div className="rounded-xl border border-border/70 bg-muted/25 p-3.5">
              <div className="mb-1.5 flex items-center justify-between">
                <span className="font-semibold text-foreground">Prompt-Driven UI Corpus</span>
                {status?.prompt_ui_dir_exists ? (
                  <Badge variant="outline" className="gap-1 border-emerald-500/40 text-emerald-500">
                    <CheckCircle2 className="size-3" /> Mounted
                  </Badge>
                ) : (
                  <Badge variant="destructive" className="gap-1">
                    <XCircle className="size-3" /> Missing
                  </Badge>
                )}
              </div>
              <code className="block break-all font-mono text-[11px] text-muted-foreground">
                {status?.prompt_ui_dir ?? "Loading..."}
              </code>
            </div>

            <div className="rounded-xl border border-border/70 bg-muted/25 p-3.5">
              <div className="mb-1.5 flex items-center justify-between">
                <span className="font-semibold text-foreground">LNC & Knowns Components</span>
                {status?.components_dir_exists ? (
                  <Badge variant="outline" className="gap-1 border-emerald-500/40 text-emerald-500">
                    <CheckCircle2 className="size-3" /> Mounted
                  </Badge>
                ) : (
                  <Badge variant="secondary" className="gap-1">
                    <XCircle className="size-3" /> Optional / Unmounted
                  </Badge>
                )}
              </div>
              <code className="block break-all font-mono text-[11px] text-muted-foreground">
                {status?.components_dir ?? settings?.components_dir ?? "—"}
              </code>
            </div>
          </CardContent>
        </Card>

        {/* Runtime Configuration (Masked) */}
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2 text-base">
              <Server className="size-4 text-primary" />
              Service Configuration (Masked)
            </CardTitle>
            <CardDescription className="text-xs">
              Runtime status from <code className="text-foreground">/api/admin/settings</code>
            </CardDescription>
          </CardHeader>
          <CardContent>
            {settings ? (
              <dl className="divide-y divide-border/50 text-xs">
                <div className="flex justify-between py-2">
                  <dt className="text-muted-foreground">Service / Runtime</dt>
                  <dd className="font-mono text-foreground">{settings.service_name} (Rust/Axum)</dd>
                </div>
                <div className="flex justify-between py-2">
                  <dt className="text-muted-foreground">Version</dt>
                  <dd className="font-mono text-foreground">{settings.service_version}</dd>
                </div>
                <div className="flex justify-between py-2">
                  <dt className="text-muted-foreground">Public Base URL</dt>
                  <dd className="font-mono text-foreground">{settings.public_base_url}</dd>
                </div>
                <div className="flex justify-between py-2">
                  <dt className="text-muted-foreground">Bind Host / Port</dt>
                  <dd className="font-mono text-foreground">
                    {settings.host}:{settings.port}
                  </dd>
                </div>
                <div className="flex justify-between py-2">
                  <dt className="text-muted-foreground">Data Root (Read-Only)</dt>
                  <dd
                    className="max-w-[260px] truncate font-mono text-foreground"
                    title={settings.data_root}
                  >
                    {settings.data_root}
                  </dd>
                </div>
                <div className="flex justify-between py-2">
                  <dt className="text-muted-foreground">OIDC SSO (lengoc)</dt>
                  <dd className="font-mono text-foreground">
                    {settings.oidc_enabled ? "Enabled (PKCE S256)" : "Not Configured"}
                  </dd>
                </div>
                <div className="flex justify-between py-2">
                  <dt className="text-muted-foreground">Management Key</dt>
                  <dd className="font-mono text-foreground">
                    {settings.api_key_configured
                      ? `Configured (${settings.api_key_masked})`
                      : "Not Set"}
                  </dd>
                </div>
                <div className="flex justify-between py-2">
                  <dt className="text-muted-foreground">Session Epoch</dt>
                  <dd className="font-mono text-foreground">
                    {settings.session_epoch ?? session?.session_epoch ?? 1}
                  </dd>
                </div>
                <div className="flex justify-between py-2">
                  <dt className="text-muted-foreground">Allowed Hosts / CORS</dt>
                  <dd className="font-mono text-foreground">
                    {settings.allowed_hosts ?? "*"} · {settings.cors_origins}
                  </dd>
                </div>
              </dl>
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
    </main>
  )
}
