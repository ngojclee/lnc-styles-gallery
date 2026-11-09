import { useState } from "react"
import { createFileRoute } from "@tanstack/react-router"
import { useQuery, useQueryClient } from "@tanstack/react-query"
import {
  CheckCircle2,
  Database,
  FileCode2,
  FolderKanban,
  HardDrive,
  Image as ImageIcon,
  KeyRound,
  Palette,
  RefreshCw,
  Server,
  Settings2,
  XCircle,
} from "lucide-react"
import { toast } from "sonner"

import {
  fetchAdminSettings,
  fetchAdminStatus,
  triggerReindex,
} from "@/api/styles"
import { PageHeader } from "@/components/common/PageHeader"
import { StatCard } from "@/components/common/StatCard"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { cn } from "@/lib/utils"

export const Route = createFileRoute("/admin")({
  component: AdminPage,
})

function AdminPage() {
  const queryClient = useQueryClient()
  const [apiKey, setApiKey] = useState("")
  const [reindexing, setReindexing] = useState(false)

  const statusQuery = useQuery({
    queryKey: ["admin-status", apiKey],
    queryFn: () => fetchAdminStatus(apiKey || undefined),
  })

  const settingsQuery = useQuery({
    queryKey: ["admin-settings", apiKey],
    queryFn: () => fetchAdminSettings(apiKey || undefined),
  })

  const status = statusQuery.data
  const settings = settingsQuery.data

  const handleReindex = async () => {
    setReindexing(true)
    try {
      const res = await triggerReindex(apiKey || undefined)
      await queryClient.invalidateQueries()
      toast.success(
        `Re-indexed ${res.summary.styles_count} styles (${res.summary.previews_count ?? 0} previews) & ${res.summary.templates_count} templates`
      )
    } catch (err) {
      toast.error(err instanceof Error ? err.message : "Re-index failed")
    } finally {
      setReindexing(false)
    }
  }

  const indexedTime = status?.indexed_at
    ? new Date(status.indexed_at * 1000).toLocaleString()
    : "—"

  return (
    <main className="flex-1 space-y-6 p-6">
      <PageHeader
        icon={Settings2}
        title="Corpus Admin & Runtime Diagnostics"
        description="Inspect read-only NAS/local corpus paths, trigger zero-downtime hot re-indexing, and review masked service settings."
        actions={
          <Button onClick={handleReindex} disabled={reindexing}>
            <RefreshCw className={cn("size-4", reindexing && "animate-spin")} />
            {reindexing ? "Scanning Corpora..." : "Trigger Hot Re-Index"}
          </Button>
        }
      />

      {/* Corpus Metrics */}
      <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
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
          hint="50 EN + 50 VI prompt-driven-ui"
          icon={FileCode2}
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
          <CardContent className="space-y-4 text-xs">
            <div className="rounded-xl border border-border/70 bg-muted/25 p-3.5">
              <div className="mb-1.5 flex items-center justify-between">
                <span className="font-semibold text-foreground">Refero Styles Corpus</span>
                {status?.refero_dir_exists ? (
                  <Badge variant="outline" className="gap-1 border-emerald-500/40 text-emerald-400">
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
                  <Badge variant="outline" className="gap-1 border-emerald-500/40 text-emerald-400">
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

            {/* Optional Admin API Key for protected mutations */}
            <div className="space-y-2 border-t border-border/60 pt-3">
              <Label htmlFor="admin-api-key" className="flex items-center gap-1.5 text-xs">
                <KeyRound className="size-3.5 text-muted-foreground" />
                Optional Admin API Key (X-API-Key)
              </Label>
              <div className="flex gap-2">
                <Input
                  id="admin-api-key"
                  type="password"
                  value={apiKey}
                  onChange={(e) => setApiKey(e.target.value)}
                  placeholder="Leave blank if STYLES_API_KEY is not enforced..."
                  className="text-xs"
                />
                <Button variant="outline" size="sm" onClick={handleReindex} disabled={reindexing}>
                  Re-Index Now
                </Button>
              </div>
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
              Environment settings from <code className="text-foreground">/api/admin/settings</code>
            </CardDescription>
          </CardHeader>
          <CardContent>
            {settings ? (
              <dl className="divide-y divide-border/50 text-xs">
                <div className="flex justify-between py-2">
                  <dt className="text-muted-foreground">Service Name</dt>
                  <dd className="font-mono text-foreground">{settings.service_name}</dd>
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
                  <dt className="text-muted-foreground">Data Root</dt>
                  <dd className="max-w-[260px] truncate font-mono text-foreground" title={settings.data_root}>
                    {settings.data_root}
                  </dd>
                </div>
                <div className="flex justify-between py-2">
                  <dt className="text-muted-foreground">API Key Configured</dt>
                  <dd className="font-mono text-foreground">
                    {settings.api_key_configured ? `Yes (${settings.api_key_masked})` : "No (Open on LAN)"}
                  </dd>
                </div>
                <div className="flex justify-between py-2">
                  <dt className="text-muted-foreground">CORS Origins</dt>
                  <dd className="font-mono text-foreground">{settings.cors_origins}</dd>
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
