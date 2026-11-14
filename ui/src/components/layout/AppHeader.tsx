import { useState } from "react"
import { Link, useRouterState } from "@tanstack/react-router"
import { useQuery, useQueryClient } from "@tanstack/react-query"
import {
  LogIn,
  LogOut,
  RefreshCw,
  Settings2,
  ShieldAlert,
  ShieldCheck,
  Shuffle,
} from "lucide-react"
import { toast } from "sonner"

import {
  authSessionQuery,
  clearAuthSessionCache,
  fetchStyles,
  healthQuery,
  logoutSession,
  revokeAllSessions,
  triggerReindex,
} from "@/api/styles"
import { AuthGateCard } from "@/components/common/AuthGateCard"
import { StyleDetailDialog } from "@/components/common/StyleDetailDialog"
import { NAV_GROUPS } from "@/components/layout/nav"
import { ThemeToggle } from "@/components/theme/ThemeToggle"
import { Badge } from "@/components/ui/badge"
import {
  Breadcrumb,
  BreadcrumbItem,
  BreadcrumbLink,
  BreadcrumbList,
  BreadcrumbPage,
  BreadcrumbSeparator,
} from "@/components/ui/breadcrumb"
import { Button } from "@/components/ui/button"
import { Dialog, DialogContent, DialogHeader, DialogTitle } from "@/components/ui/dialog"
import { Separator } from "@/components/ui/separator"
import { SidebarTrigger } from "@/components/ui/sidebar"
import { cn } from "@/lib/utils"

export function AppHeader() {
  const pathname = useRouterState({ select: (s) => s.location.pathname })
  const queryClient = useQueryClient()
  const { data: health } = useQuery(healthQuery)
  const { data: session } = useQuery(authSessionQuery)

  const [reindexing, setReindexing] = useState(false)
  const [randomStyleId, setRandomStyleId] = useState<string | null>(null)
  const [randomOpen, setRandomOpen] = useState(false)
  const [authModalOpen, setAuthModalOpen] = useState(false)
  const [sessionDialogOpen, setSessionDialogOpen] = useState(false)
  const [busySessionAction, setBusySessionAction] = useState(false)

  const activeNav = NAV_GROUPS.flatMap((g) => g.items).find((item) => {
    if (item.to === "/styles") return pathname === "/" || pathname.startsWith("/styles")
    if (item.to === "/keys") {
      return (
        pathname === "/keys" ||
        pathname.startsWith("/keys/") ||
        pathname === "/api-keys" ||
        pathname.startsWith("/api-keys/")
      )
    }
    if (item.to === "/settings") {
      return (
        pathname === "/settings" ||
        pathname.startsWith("/settings/") ||
        pathname === "/admin" ||
        pathname.startsWith("/admin/")
      )
    }
    return pathname === item.to || pathname.startsWith(`${item.to}/`)
  })

  const handleReindex = async () => {
    if (session?.auth_required && !session.authenticated) {
      setAuthModalOpen(true)
      return
    }
    setReindexing(true)
    try {
      const res = await triggerReindex()
      await queryClient.invalidateQueries()
      toast.success(
        `Corpus re-indexed: ${res.summary.styles_count} styles, ${res.summary.templates_count} templates`
      )
    } catch (err) {
      toast.error(err instanceof Error ? err.message : "Re-index failed")
    } finally {
      setReindexing(false)
    }
  }

  const handleRandomStyle = async () => {
    try {
      const total = health?.styles_count ?? 200
      const randomOffset = Math.max(0, Math.floor(Math.random() * total))
      const list = await fetchStyles({ limit: 1, offset: randomOffset })
      const picked = list.items[0]
      if (picked) {
        setRandomStyleId(picked.id)
        setRandomOpen(true)
      } else {
        toast.info("No styles found in corpus")
      }
    } catch {
      toast.error("Failed to pick a random style")
    }
  }

  const handleLogout = async () => {
    setBusySessionAction(true)
    try {
      await logoutSession()
      setSessionDialogOpen(false)
      clearAuthSessionCache(queryClient)
      await queryClient.invalidateQueries({ queryKey: authSessionQuery.queryKey })
      toast.success("Signed out")
    } catch (err) {
      toast.error(err instanceof Error ? err.message : "Failed to sign out")
    } finally {
      setBusySessionAction(false)
    }
  }

  const handleRevokeAll = async () => {
    setBusySessionAction(true)
    try {
      const res = await revokeAllSessions()
      setSessionDialogOpen(false)
      clearAuthSessionCache(queryClient)
      await queryClient.invalidateQueries({ queryKey: authSessionQuery.queryKey })
      toast.success(`All sessions revoked (epoch ${res.session_epoch})`)
    } catch (err) {
      toast.error(err instanceof Error ? err.message : "Failed to revoke sessions")
    } finally {
      setBusySessionAction(false)
    }
  }

  const identityLabel =
    session?.user?.username ||
    session?.user?.email ||
    session?.sub ||
    (session?.mode === "mgmt_key" ? "mgmt" : "Session")

  return (
    <>
      <header className="sticky top-0 z-20 flex h-14 shrink-0 items-center justify-between gap-2 border-b border-border/60 bg-background/85 px-4 backdrop-blur-md">
        <div className="flex items-center gap-2">
          <SidebarTrigger className="-ml-1" />
          <Separator orientation="vertical" className="mr-2 h-4" />
          <Breadcrumb>
            <BreadcrumbList>
              <BreadcrumbItem className="hidden md:block">
                <BreadcrumbLink asChild>
                  <Link to="/styles">LNC Style Library</Link>
                </BreadcrumbLink>
              </BreadcrumbItem>
              <BreadcrumbSeparator className="hidden md:block" />
              <BreadcrumbItem>
                <BreadcrumbPage>{activeNav?.title ?? "Design Styles"}</BreadcrumbPage>
              </BreadcrumbItem>
            </BreadcrumbList>
          </Breadcrumb>
        </div>

        <div className="flex items-center gap-2">
          <Button
            variant="outline"
            size="sm"
            onClick={handleRandomStyle}
            title="Inspect a random design style"
          >
            <Shuffle className="size-3.5" />
            <span className="hidden sm:inline">Random Style</span>
          </Button>

          <Button
            variant="outline"
            size="sm"
            onClick={handleReindex}
            disabled={reindexing}
            title="Hot re-scan corpus directories"
          >
            <RefreshCw className={cn("size-3.5", reindexing && "animate-spin")} />
            <span className="hidden sm:inline">{reindexing ? "Re-indexing..." : "Re-Index"}</span>
          </Button>

          <Button
            variant="ghost"
            size="sm"
            asChild
            title="Corpus Admin & Diagnostics"
          >
            <Link to="/admin">
              <Settings2 className="size-3.5" />
              <span className="hidden md:inline">Admin</span>
            </Link>
          </Button>

          {session?.authenticated ? (
            <Button
              variant="outline"
              size="sm"
              className="gap-1.5 border-emerald-500/35 text-xs"
              onClick={() => setSessionDialogOpen(true)}
              title="Manage active authenticated session"
            >
              <ShieldCheck className="size-3.5 text-emerald-500" />
              <span className="hidden md:inline max-w-[120px] truncate">{identityLabel}</span>
            </Button>
          ) : session?.auth_required ? (
            <Button
              variant="outline"
              size="sm"
              className="gap-1.5 text-xs"
              onClick={() => setAuthModalOpen(true)}
              title="Sign in with LNC SSO or management key"
            >
              <LogIn className="size-3.5" />
              <span className="hidden sm:inline">Sign In</span>
            </Button>
          ) : null}

          <ThemeToggle />
        </div>
      </header>

      <StyleDetailDialog
        styleId={randomStyleId}
        open={randomOpen}
        onOpenChange={setRandomOpen}
      />

      <Dialog open={authModalOpen} onOpenChange={setAuthModalOpen}>
        <DialogContent className="max-w-md">
          <DialogHeader className="sr-only">
            <DialogTitle>Sign in to LNC Style Gallery</DialogTitle>
          </DialogHeader>
          <AuthGateCard
            nextPath={pathname || "/keys"}
            oidcEnabled={session?.oidc_enabled ?? true}
            mgmtKeyConfigured={session?.mgmt_key_configured ?? true}
            compact
            onAuthenticated={() => setAuthModalOpen(false)}
          />
        </DialogContent>
      </Dialog>

      <Dialog open={sessionDialogOpen} onOpenChange={setSessionDialogOpen}>
        <DialogContent className="max-w-sm">
          <DialogHeader>
            <DialogTitle className="flex items-center gap-2 text-base">
              <ShieldCheck className="size-4 text-emerald-500" />
              Active Console Session
            </DialogTitle>
          </DialogHeader>
          <div className="space-y-4 text-xs">
            <div className="rounded-xl border border-border/70 bg-muted/30 p-3 space-y-1.5">
              <div className="flex items-center justify-between">
                <span className="text-muted-foreground">Identity</span>
                <span className="font-mono font-semibold text-foreground">{identityLabel}</span>
              </div>
              <div className="flex items-center justify-between">
                <span className="text-muted-foreground">Auth Mode</span>
                <Badge variant="secondary" className="text-[10px] uppercase">
                  {session?.mode ?? "session"}
                </Badge>
              </div>
              <div className="flex items-center justify-between">
                <span className="text-muted-foreground">Session Epoch</span>
                <span className="font-mono text-foreground">{session?.session_epoch ?? 1}</span>
              </div>
            </div>

            <div className="flex flex-col gap-2">
              <Button
                variant="outline"
                size="sm"
                className="w-full justify-center gap-1.5"
                disabled={busySessionAction}
                onClick={handleLogout}
              >
                <LogOut className="size-3.5" />
                <span>Sign Out This Browser</span>
              </Button>
              <Button
                variant="destructive"
                size="sm"
                className="w-full justify-center gap-1.5"
                disabled={busySessionAction}
                onClick={handleRevokeAll}
              >
                <ShieldAlert className="size-3.5" />
                <span>Revoke All Sessions</span>
              </Button>
            </div>
          </div>
        </DialogContent>
      </Dialog>
    </>
  )
}
