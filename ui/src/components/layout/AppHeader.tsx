import { useState } from "react"
import { Link, useRouterState } from "@tanstack/react-router"
import { useQuery, useQueryClient } from "@tanstack/react-query"
import { RefreshCw, Settings2, Shuffle } from "lucide-react"
import { toast } from "sonner"

import { fetchStyles, healthQuery, triggerReindex } from "@/api/styles"
import { StyleDetailDialog } from "@/components/common/StyleDetailDialog"
import { NAV_GROUPS } from "@/components/layout/nav"
import { ThemeToggle } from "@/components/theme/ThemeToggle"
import {
  Breadcrumb,
  BreadcrumbItem,
  BreadcrumbLink,
  BreadcrumbList,
  BreadcrumbPage,
  BreadcrumbSeparator,
} from "@/components/ui/breadcrumb"
import { Button } from "@/components/ui/button"
import { Separator } from "@/components/ui/separator"
import { SidebarTrigger } from "@/components/ui/sidebar"
import { cn } from "@/lib/utils"

export function AppHeader() {
  const pathname = useRouterState({ select: (s) => s.location.pathname })
  const queryClient = useQueryClient()
  const { data: health } = useQuery(healthQuery)

  const [reindexing, setReindexing] = useState(false)
  const [randomStyleId, setRandomStyleId] = useState<string | null>(null)
  const [randomOpen, setRandomOpen] = useState(false)

  const activeNav = NAV_GROUPS.flatMap((g) => g.items).find((item) => {
    if (item.to === "/styles") return pathname === "/" || pathname.startsWith("/styles")
    return pathname === item.to || pathname.startsWith(`${item.to}/`)
  })

  const handleReindex = async () => {
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

          <ThemeToggle />
        </div>
      </header>

      <StyleDetailDialog
        styleId={randomStyleId}
        open={randomOpen}
        onOpenChange={setRandomOpen}
      />
    </>
  )
}
