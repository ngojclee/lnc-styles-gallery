import { createFileRoute } from '@tanstack/react-router'
import {
  AlertTriangleIcon,
  BanIcon,
  CheckCircle2Icon,
  CheckIcon,
  CopyIcon,
  CpuIcon,
  EyeIcon,
  EyeOffIcon,
  KeyRoundIcon,
  PlusIcon,
  RefreshCwIcon,
  RotateCcwIcon,
  ServerIcon,
  ShieldCheckIcon,
  Trash2Icon,
} from 'lucide-react'
import { useEffect, useState } from 'react'
import { toast } from '@/lib/notifications'
import { PageHeader } from '@/components/common/PageHeader'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Dialog, DialogContent, DialogHeader, DialogTitle } from '@/components/ui/dialog'
import { Input } from '@/components/ui/input'

export const Route = createFileRoute('/keys')({
  staticData: { title: 'API Keys & MCP Access' },
  component: ApiKeysPage,
})

interface ApiKeyItem {
  id: string
  label: string
  prefix: string
  raw_key?: string
  enabled: boolean
  created: string
  rotated_at?: string
}

interface McpInfo {
  endpoint: string
  namespace: string
  transport: string
  auth_required: boolean
  tools: { name: string; description: string }[]
  tools_count: number
}

function ApiKeysPage() {
  const [keys, setKeys] = useState<ApiKeyItem[]>([])
  const [mcpInfo, setMcpInfo] = useState<McpInfo | null>(null)
  const [loading, setLoading] = useState(true)
  const [isCreateOpen, setIsCreateOpen] = useState(false)
  const [label, setLabel] = useState('')
  const [customSecret, setCustomSecret] = useState('')
  const [adminKey, setAdminKey] = useState('')
  const [createdSecret, setCreatedSecret] = useState<string | null>(null)
  const [copiedKey, setCopiedKey] = useState<string | null>(null)
  const [submitting, setSubmitting] = useState(false)
  const [revealedKeys, setRevealedKeys] = useState<Record<string, boolean>>({})
  const [rotateDialog, setRotateDialog] = useState<{
    open: boolean
    keyId: string
    label: string
    newKey?: string
    rotating?: boolean
  } | null>(null)

  const adminHeaders = () => {
    const h: Record<string, string> = { 'Content-Type': 'application/json' }
    if (adminKey.trim()) h['X-API-Key'] = adminKey.trim()
    return h
  }

  const fetchKeys = async () => {
    try {
      setLoading(true)
      const res = await fetch('/api/keys')
      const data = await res.json()
      if (data.ok && Array.isArray(data.keys)) setKeys(data.keys)
    } catch {
      toast.error('Failed to load API keys')
    } finally {
      setLoading(false)
    }
  }

  const fetchMcpInfo = async () => {
    try {
      const res = await fetch('/api/mcp/info')
      if (res.ok) setMcpInfo(await res.json())
    } catch { /* ignore */ }
  }

  useEffect(() => {
    fetchKeys()
    fetchMcpInfo()
  }, [])

  const toggleVisibility = (id: string) =>
    setRevealedKeys((prev) => ({ ...prev, [id]: !prev[id] }))

  const copyToClipboard = (text: string, id: string) => {
    navigator.clipboard.writeText(text)
    setCopiedKey(id)
    toast.success('Copied to clipboard!')
    setTimeout(() => setCopiedKey(null), 2000)
  }

  const handleCreate = async (e: React.FormEvent) => {
    e.preventDefault()
    if (!label.trim()) {
      toast.error('Please provide a key name or label')
      return
    }
    setSubmitting(true)
    try {
      const res = await fetch('/api/keys', {
        method: 'POST',
        headers: adminHeaders(),
        body: JSON.stringify({ label: label.trim(), custom_secret: customSecret.trim() || undefined }),
      })
      const data = await res.json()
      if (data.ok && data.key) {
        setCreatedSecret(data.key)
        setLabel('')
        setCustomSecret('')
        toast.success('API Key generated successfully!')
        fetchKeys()
      } else {
        toast.error(data.detail || 'Failed to create key')
      }
    } catch {
      toast.error('Failed to create key')
    } finally {
      setSubmitting(false)
    }
  }

  const handleRevoke = async (keyId: string, revoke: boolean) => {
    try {
      const res = await fetch(`/api/keys/${keyId}/revoke`, {
        method: 'POST',
        headers: adminHeaders(),
        body: JSON.stringify({ revoke }),
      })
      const data = await res.json()
      if (data.ok) {
        toast.success(revoke ? 'API Key revoked' : 'API Key reactivated')
        fetchKeys()
      } else {
        toast.error(data.detail || 'Failed to update key')
      }
    } catch {
      toast.error('Failed to update key')
    }
  }

  const handleExecuteRotate = async () => {
    if (!rotateDialog) return
    setRotateDialog((prev) => (prev ? { ...prev, rotating: true } : null))
    try {
      const res = await fetch(`/api/keys/${rotateDialog.keyId}/rotate`, {
        method: 'POST',
        headers: adminHeaders(),
        body: JSON.stringify({}),
      })
      const data = await res.json()
      if (data.ok && data.key) {
        setRotateDialog((prev) => (prev ? { ...prev, rotating: false, newKey: data.key } : null))
        toast.success(`Key "${rotateDialog.label}" rotated successfully!`)
        fetchKeys()
      } else {
        toast.error(data.detail || 'Failed to rotate key')
        setRotateDialog((prev) => (prev ? { ...prev, rotating: false } : null))
      }
    } catch {
      toast.error('Failed to rotate key')
      setRotateDialog((prev) => (prev ? { ...prev, rotating: false } : null))
    }
  }

  const handleDelete = async (keyId: string, keyLabel: string) => {
    if (!window.confirm(`Permanently DELETE key "${keyLabel}"? Any connected agents using this token will fail immediately.`)) return
    try {
      const res = await fetch(`/api/keys/${keyId}`, { method: 'DELETE', headers: adminHeaders() })
      const data = await res.json()
      if (data.ok) {
        toast.success(`Key "${keyLabel}" permanently deleted`)
        fetchKeys()
      } else {
        toast.error(data.detail || 'Failed to delete key')
      }
    } catch {
      toast.error('Failed to delete key')
    }
  }

  const renderMaskedKey = (rawKey: string, isVisible: boolean) => {
    if (isVisible) return rawKey
    if (rawKey.startsWith('styles_')) return 'styles_••••••••••••••••••••••'
    return '••••••••••••••••••••••••••••••••'
  }

  const endpoint = mcpInfo?.endpoint ?? 'https://styles.lengoc.me/mcp'
  const namespace = mcpInfo?.namespace ?? 'styles__*'

  return (
    <div className="flex flex-col gap-6">
      <PageHeader
        title="API Keys & MCP Access"
        description="Manage Bearer tokens for external agents, FastMCP client connections, and Business MCP Hub integration. Tokens are masked by default for security."
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
            <span className="text-xs font-semibold uppercase tracking-wider">FastMCP Endpoint</span>
          </div>
          <div className="mt-2 font-mono text-sm font-semibold truncate text-foreground select-all">{endpoint}</div>
          <p className="mt-1 text-xs text-muted-foreground">{mcpInfo?.transport ?? 'Streamable HTTP JSON-RPC 2.0 & SSE transport'}</p>
        </div>

        <div className="rounded-xl border bg-card p-4 shadow-xs">
          <div className="flex items-center gap-2 text-muted-foreground">
            <ShieldCheckIcon className="size-4 text-emerald-500" />
            <span className="text-xs font-semibold uppercase tracking-wider">Authentication</span>
          </div>
          <div className="mt-2 text-sm font-semibold text-foreground">
            {mcpInfo?.auth_required ? 'Bearer Token Required' : 'Open — Internal Network'}
          </div>
          <p className="mt-1 text-xs text-muted-foreground">
            {mcpInfo?.auth_required
              ? 'Include header: Authorization: Bearer <key>'
              : 'Optional — enable Bearer gate via STYLES_MCP_AUTH=1'}
          </p>
        </div>

        <div className="rounded-xl border bg-card p-4 shadow-xs">
          <div className="flex items-center gap-2 text-muted-foreground">
            <CpuIcon className="size-4 text-blue-500" />
            <span className="text-xs font-semibold uppercase tracking-wider">Business MCP Hub</span>
          </div>
          <div className="mt-2 text-sm font-semibold text-foreground">
            Online · {mcpInfo?.tools_count ?? '…'} Tools Active
          </div>
          <p className="mt-1 text-xs text-muted-foreground">Exposes {namespace} tools across fleet agents</p>
        </div>
      </div>

      {/* Keys List */}
      <div className="rounded-2xl border bg-card overflow-hidden shadow-xs">
        <div className="flex items-center justify-between border-b px-5 py-3.5 bg-muted/30">
          <div className="flex items-center gap-2">
            <KeyRoundIcon className="size-4 text-primary" />
            <h3 className="font-semibold text-sm">Active API Tokens</h3>
          </div>
          <div className="flex items-center gap-2">
            <Input
              value={adminKey}
              onChange={(e) => setAdminKey(e.target.value)}
              placeholder="Admin X-API-Key (if set)"
              type="password"
              className="h-8 w-48 text-xs"
            />
            <Button variant="ghost" size="sm" onClick={fetchKeys} className="h-8 gap-1 text-xs text-muted-foreground">
              <RotateCcwIcon className="size-3.5" />
              <span>Refresh</span>
            </Button>
          </div>
        </div>

        <div className="divide-y">
          {loading ? (
            <div className="p-8 text-center text-sm text-muted-foreground">Loading API keys...</div>
          ) : keys.length === 0 ? (
            <div className="p-8 text-center text-sm text-muted-foreground">
              No API keys configured yet. Click "Generate New Key" to create your first token.
            </div>
          ) : (
            keys.map((k) => {
              const fullKey = k.raw_key || k.prefix
              const isVisible = !!revealedKeys[k.id]
              return (
                <div key={k.id} className="flex flex-col lg:flex-row lg:items-center justify-between p-4 gap-4">
                  <div className="flex flex-col gap-1.5 flex-1 min-w-0">
                    <div className="flex items-center gap-2 flex-wrap">
                      <span className="font-semibold text-sm text-foreground">{k.label}</span>
                      <Badge
                        variant={k.enabled ? 'outline' : 'secondary'}
                        className={`text-[10px] ${
                          k.enabled
                            ? 'border-emerald-500/40 text-emerald-600 dark:text-emerald-400 bg-emerald-500/10'
                            : 'border-destructive/30 text-destructive bg-destructive/10'
                        }`}
                      >
                        {k.enabled ? 'Active' : 'Revoked / Disabled'}
                      </Badge>
                    </div>

                    <div className="flex items-center gap-2 flex-wrap text-xs text-muted-foreground">
                      <span className="font-medium text-xs">Token:</span>
                      <div className="inline-flex items-center gap-1.5 bg-muted/80 border rounded-md px-2 py-1 font-mono text-xs text-foreground">
                        <span className="tracking-wide select-all">{renderMaskedKey(fullKey, isVisible)}</span>
                        <Button
                          variant="ghost"
                          size="icon"
                          className="h-5 w-5 text-muted-foreground hover:text-foreground shrink-0"
                          onClick={() => toggleVisibility(k.id)}
                          title={isVisible ? 'Hide token' : 'Reveal token'}
                        >
                          {isVisible ? <EyeOffIcon className="size-3.5" /> : <EyeIcon className="size-3.5" />}
                        </Button>
                      </div>
                      {k.created && (
                        <span className="text-[11px] text-muted-foreground ml-1">
                          Created: {new Date(k.created).toLocaleDateString()}
                        </span>
                      )}
                      {k.rotated_at && (
                        <span className="text-[11px] text-muted-foreground">
                          · Rotated: {new Date(k.rotated_at).toLocaleDateString()}
                        </span>
                      )}
                    </div>
                  </div>

                  <div className="flex items-center gap-1.5 flex-wrap self-start lg:self-center shrink-0">
                    <Button variant="outline" size="sm" className="h-8 gap-1 text-xs" onClick={() => copyToClipboard(fullKey, k.id)} title="Copy full token to clipboard">
                      {copiedKey === k.id ? <CheckIcon className="size-3.5 text-emerald-500" /> : <CopyIcon className="size-3.5" />}
                      <span>Copy</span>
                    </Button>
                    <Button
                      variant="outline"
                      size="sm"
                      className="h-8 gap-1 text-xs text-blue-600 dark:text-blue-400 hover:bg-blue-50 dark:hover:bg-blue-950/40"
                      onClick={() => setRotateDialog({ open: true, keyId: k.id, label: k.label })}
                      title="Rotate key and generate a fresh token"
                    >
                      <RefreshCwIcon className="size-3.5" />
                      <span>Rotate</span>
                    </Button>
                    {k.enabled ? (
                      <Button
                        variant="outline"
                        size="sm"
                        className="h-8 gap-1 text-xs text-amber-600 dark:text-amber-400 hover:bg-amber-50 dark:hover:bg-amber-950/40"
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
                        className="h-8 gap-1 text-xs text-emerald-600 dark:text-emerald-400 hover:bg-emerald-50 dark:hover:bg-emerald-950/40"
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
              )
            })
          )}
        </div>
      </div>

      {/* Integration Guide */}
      <div className="rounded-2xl border bg-muted/40 p-5 flex flex-col gap-4 text-xs">
        <h3 className="font-semibold text-sm text-foreground">How to Connect from AI Agents & MCP Clients</h3>
        <div className="grid gap-4 md:grid-cols-2">
          <div className="flex flex-col gap-2 rounded-xl border bg-card p-4">
            <h4 className="font-semibold text-xs text-foreground">1. Central Business MCP Hub Registration</h4>
            <p className="text-muted-foreground leading-relaxed">
              Connected into the central Business MCP Hub exposing tools with prefix{' '}
              <code className="font-semibold text-foreground">{namespace}</code>.
            </p>
            <pre className="rounded-lg bg-neutral-950 p-3 text-neutral-300 font-mono text-[11px] overflow-x-auto leading-relaxed">
{`# Connect directly to Streamable HTTP MCP
POST ${endpoint}
Headers:
  Authorization: Bearer <YOUR_API_KEY>
  Content-Type: application/json
  Accept: application/json, text/event-stream`}
            </pre>
          </div>

          <div className="flex flex-col gap-2 rounded-xl border bg-card p-4">
            <h4 className="font-semibold text-xs text-foreground">2. Available FastMCP Tools</h4>
            <ul className="list-disc pl-4 space-y-1 text-muted-foreground">
              {(mcpInfo?.tools ?? []).map((t) => (
                <li key={t.name}>
                  <code className="text-foreground font-semibold">styles__{t.name}</code>: {t.description.split('.')[0]}.
                </li>
              ))}
              {!mcpInfo && <li>Loading tool list…</li>}
            </ul>
          </div>
        </div>
      </div>

      {/* Create Key Dialog */}
      <Dialog open={isCreateOpen} onOpenChange={(o) => { if (!o) { setIsCreateOpen(false); setCreatedSecret(null) } }}>
        <DialogContent className="max-w-md">
          <DialogHeader>
            <DialogTitle>Generate New API Key</DialogTitle>
          </DialogHeader>
          {createdSecret ? (
            <div className="flex flex-col gap-4 my-2">
              <div className="rounded-lg bg-emerald-500/10 border border-emerald-500/30 p-3 text-xs text-emerald-800 dark:text-emerald-300">
                Key generated successfully! Copy this key now as it will be masked on the main list.
              </div>
              <div className="flex items-center gap-2 rounded-xl bg-muted p-3 font-mono text-xs break-all">
                <span className="flex-1 select-all font-semibold text-foreground">{createdSecret}</span>
                <Button size="sm" variant="outline" className="shrink-0 h-8 gap-1 text-xs" onClick={() => copyToClipboard(createdSecret, 'newly_created')}>
                  {copiedKey === 'newly_created' ? <CheckIcon className="size-3.5 text-emerald-500" /> : <CopyIcon className="size-3.5" />}
                  <span>Copy</span>
                </Button>
              </div>
              <Button onClick={() => { setIsCreateOpen(false); setCreatedSecret(null) }} className="w-full">Done</Button>
            </div>
          ) : (
            <form onSubmit={handleCreate} className="flex flex-col gap-4 my-2">
              <div className="flex flex-col gap-1.5">
                <label className="text-xs font-semibold text-foreground">Key Name / Client Label</label>
                <Input value={label} onChange={(e) => setLabel(e.target.value)} placeholder="e.g. Business MCP Hub, Agent Hermes" required />
              </div>
              <div className="flex flex-col gap-1.5">
                <label className="text-xs font-semibold text-foreground">Custom Secret (Optional)</label>
                <Input value={customSecret} onChange={(e) => setCustomSecret(e.target.value)} placeholder="Leave empty to auto-generate a secure random token" />
                <span className="text-[11px] text-muted-foreground">
                  If empty, a cryptographically secure token starting with <code className="font-mono">styles_...</code> will be generated.
                </span>
              </div>
              <div className="flex justify-end gap-2 mt-2">
                <Button type="button" variant="outline" onClick={() => setIsCreateOpen(false)}>Cancel</Button>
                <Button type="submit" disabled={submitting}>{submitting ? 'Generating...' : 'Generate API Key'}</Button>
              </div>
            </form>
          )}
        </DialogContent>
      </Dialog>

      {/* Rotate Key Dialog */}
      <Dialog open={!!rotateDialog?.open} onOpenChange={(o) => { if (!o) setRotateDialog(null) }}>
        <DialogContent className="max-w-md">
          <DialogHeader>
            <DialogTitle>Rotate API Key</DialogTitle>
          </DialogHeader>
          {rotateDialog?.newKey ? (
            <div className="flex flex-col gap-4 my-2">
              <div className="rounded-lg bg-emerald-500/10 border border-emerald-500/30 p-3 text-xs text-emerald-800 dark:text-emerald-300">
                Key rotated successfully! The old secret has been replaced with this new token.
              </div>
              <div className="flex items-center gap-2 rounded-xl bg-muted p-3 font-mono text-xs break-all">
                <span className="flex-1 select-all font-semibold text-foreground">{rotateDialog.newKey}</span>
                <Button size="sm" variant="outline" className="shrink-0 h-8 gap-1 text-xs" onClick={() => copyToClipboard(rotateDialog.newKey!, 'rotated_key')}>
                  {copiedKey === 'rotated_key' ? <CheckIcon className="size-3.5 text-emerald-500" /> : <CopyIcon className="size-3.5" />}
                  <span>Copy</span>
                </Button>
              </div>
              <Button onClick={() => setRotateDialog(null)} className="w-full">Done</Button>
            </div>
          ) : (
            <div className="flex flex-col gap-4 my-2">
              <div className="flex items-start gap-3 rounded-lg bg-amber-500/10 border border-amber-500/30 p-3 text-xs text-amber-800 dark:text-amber-300">
                <AlertTriangleIcon className="size-4 shrink-0 text-amber-600 mt-0.5" />
                <span>
                  Rotating key <strong className="font-semibold">"{rotateDialog?.label}"</strong> will invalidate the existing secret immediately. Any client or agent using the old token will need to be updated.
                </span>
              </div>
              <div className="flex justify-end gap-2 mt-2">
                <Button type="button" variant="outline" onClick={() => setRotateDialog(null)}>Cancel</Button>
                <Button variant="default" onClick={handleExecuteRotate} disabled={rotateDialog?.rotating} className="bg-blue-600 hover:bg-blue-700 text-white gap-1.5">
                  <RefreshCwIcon className={`size-3.5 ${rotateDialog?.rotating ? 'animate-spin' : ''}`} />
                  <span>{rotateDialog?.rotating ? 'Rotating...' : 'Rotate Key Now'}</span>
                </Button>
              </div>
            </div>
          )}
        </DialogContent>
      </Dialog>
    </div>
  )
}
