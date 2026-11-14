import { useEffect, useMemo, useState } from "react"
import { createFileRoute } from "@tanstack/react-router"
import { useQuery } from "@tanstack/react-query"
import {
  Blocks,
  Check,
  Copy,
  ExternalLink,
  Eye,
  FileCode2,
  Search,
  X,
} from "lucide-react"
import { toast } from "sonner"

import { fetchComponentDetail, fetchComponents } from "@/api/styles"
import { EmptyState } from "@/components/common/EmptyState"
import { PageHeader } from "@/components/common/PageHeader"
import { StatCard } from "@/components/common/StatCard"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { Card } from "@/components/ui/card"
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog"
import { Input } from "@/components/ui/input"
import { copyText } from "@/lib/clipboard"
import { Skeleton } from "@/components/ui/skeleton"
import { cn } from "@/lib/utils"
import type { ComponentItem } from "@/types/styles"

export const Route = createFileRoute("/components")({
  component: ComponentsPage,
})

const CATEGORY_LABELS: Record<string, string> = {
  backgrounds: "Backgrounds",
  buttons: "Buttons",
  cards: "Cards",
  loaders: "Loaders",
  "inputs-toggles": "Inputs & Toggles",
  "text-effects": "Text Effects",
  "ai-agent": "AI Agent",
  marketing: "Marketing",
  "knowns-ui": "Knowns UI",
  misc: "Misc",
}

function ComponentsPage() {
  const [catFilter, setCatFilter] = useState<string>("all")
  const [kindFilter, setKindFilter] = useState<string>("all")
  const [searchInput, setSearchInput] = useState("")
  const [debouncedQuery, setDebouncedQuery] = useState("")
  const [selected, setSelected] = useState<ComponentItem | null>(null)
  const [dialogOpen, setDialogOpen] = useState(false)
  const [dialogTab, setDialogTab] = useState<"preview" | "code">("preview")
  const [copied, setCopied] = useState(false)

  useEffect(() => {
    const timer = setTimeout(() => setDebouncedQuery(searchInput.trim()), 220)
    return () => clearTimeout(timer)
  }, [searchInput])

  const componentsQuery = useQuery({
    queryKey: [
      "components",
      {
        category: catFilter === "all" ? undefined : catFilter,
        kind: kindFilter === "all" ? undefined : kindFilter,
        q: debouncedQuery,
      },
    ],
    queryFn: () =>
      fetchComponents({
        category: catFilter === "all" ? undefined : catFilter,
        kind: kindFilter === "all" ? undefined : kindFilter,
        q: debouncedQuery || undefined,
        limit: 500,
      }),
  })

  const detailQuery = useQuery({
    queryKey: ["component-detail", selected?.id],
    queryFn: () => fetchComponentDetail(selected!.id, true),
    enabled: Boolean(dialogOpen && selected?.id),
  })

  const items = componentsQuery.data?.items ?? []
  const catFacets = componentsQuery.data?.facets?.categories ?? []
  const lncCount = items.filter((i) => i.kind === "html").length
  const knownsCount = items.filter((i) => i.kind === "tsx").length
  const total = componentsQuery.data?.total ?? 0

  const catOptions = useMemo(
    () => [{ id: "all", count: total }, ...catFacets],
    [catFacets, total]
  )

  const openComponent = (c: ComponentItem) => {
    setSelected(c)
    setDialogTab(c.kind === "html" ? "preview" : "code")
    setCopied(false)
    setDialogOpen(true)
  }

  const sourceText = useMemo(() => {
    const d = detailQuery.data
    if (!d?.sources) return ""
    return Object.entries(d.sources)
      .map(([name, code]) => `// ── ${name} ──\n${code}`)
      .join("\n\n")
  }, [detailQuery.data])

  const handleCopySource = async () => {
    const text = sourceText
    if (!text) {
      toast.error("No source available to copy")
      return
    }
    const ok = await copyText(text)
    if (ok) {
      setCopied(true)
      toast.success(`Copied source for ${selected?.id}`)
      setTimeout(() => setCopied(false), 2000)
    } else {
      toast.error("Clipboard copy failed")
    }
  }

  return (
    <main className="flex-1 space-y-6">
      <PageHeader
        icon={Blocks}
        title="Components & Effects"
        description="UI effect recipes and component sources — LNC self-contained HTML demos (live preview) plus the MIT-licensed Knowns UI kit for AI products (TSX source)."
      />

      <div className="grid gap-4 sm:grid-cols-3">
        <StatCard
          label="Total Components"
          value={total}
          hint="Effects, components & blocks"
          icon={Blocks}
        />
        <StatCard
          label="LNC Live Demos"
          value={lncCount}
          hint="Self-contained HTML/CSS/JS — iframe preview"
          icon={Eye}
        />
        <StatCard
          label="Knowns UI (MIT)"
          value={knownsCount}
          hint="React/TSX sources for AI products"
          icon={FileCode2}
        />
      </div>

      <Card className="flex flex-col gap-3 p-4">
        <div className="flex flex-col gap-3 lg:flex-row lg:items-center">
          <div className="relative flex-1">
            <Search className="pointer-events-none absolute top-1/2 left-3 size-4 -translate-y-1/2 text-muted-foreground" />
            <Input
              value={searchInput}
              onChange={(e) => setSearchInput(e.target.value)}
              placeholder="Search components by name, effect, or tag..."
              className="pl-9 pr-8"
            />
            {searchInput ? (
              <button
                type="button"
                onClick={() => setSearchInput("")}
                className="absolute top-1/2 right-2.5 -translate-y-1/2 text-muted-foreground hover:text-foreground"
              >
                <X className="size-3.5" />
              </button>
            ) : null}
          </div>

          <div className="flex items-center gap-1.5">
            {(["all", "html", "tsx"] as const).map((k) => (
              <Button
                key={k}
                variant={kindFilter === k ? "default" : "outline"}
                size="sm"
                onClick={() => setKindFilter(k)}
              >
                {k === "all" ? "All Kinds" : k === "html" ? "Live HTML" : "TSX Source"}
              </Button>
            ))}
          </div>
        </div>

        <div className="flex flex-wrap items-center gap-1.5">
          {catOptions.map((cat) => (
            <Button
              key={cat.id}
              variant={catFilter === cat.id ? "secondary" : "ghost"}
              size="xs"
              onClick={() => setCatFilter(cat.id)}
              className="text-xs"
            >
              {cat.id === "all" ? "All" : (CATEGORY_LABELS[cat.id] ?? cat.id)}
              <span className="ml-1 text-muted-foreground">{cat.count}</span>
            </Button>
          ))}
        </div>
      </Card>

      {componentsQuery.isLoading ? (
        <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
          {Array.from({ length: 8 }).map((_, idx) => (
            <Card key={idx} className="overflow-hidden p-0">
              <Skeleton className="aspect-[16/10] w-full rounded-none" />
              <div className="space-y-2 p-4">
                <Skeleton className="h-5 w-3/4" />
                <Skeleton className="h-3.5 w-full" />
              </div>
            </Card>
          ))}
        </div>
      ) : items.length === 0 ? (
        <EmptyState
          icon={Blocks}
          title="No matching components"
          description="Try clearing your search or switching category."
        />
      ) : (
        <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
          {items.map((c) => (
            <Card
              key={c.id}
              onClick={() => openComponent(c)}
              className="group cursor-pointer gap-0 overflow-hidden p-0 transition-all hover:-translate-y-0.5 hover:border-primary/50 hover:shadow-lg"
            >
              <div className="relative aspect-[16/9] w-full overflow-hidden border-b border-border/60 bg-zinc-950">
                {c.has_preview && c.preview_url ? (
                  <img
                    src={c.preview_url}
                    alt={c.name}
                    loading="lazy"
                    className="h-full w-full object-cover transition-transform duration-300 group-hover:scale-[1.03]"
                  />
                ) : c.has_html && c.html_url ? (
                  <iframe
                    src={c.html_url}
                    title={c.name}
                    loading="lazy"
                    sandbox="allow-scripts"
                    className="pointer-events-none h-full w-full border-0"
                  />
                ) : (
                  <div className="flex h-full w-full flex-col items-center justify-center gap-2 text-muted-foreground">
                    <FileCode2 className="size-8 opacity-40" />
                    <span className="font-mono text-[10px] tracking-wider uppercase">
                      {c.files.length} source file{c.files.length === 1 ? "" : "s"}
                    </span>
                  </div>
                )}
                <div className="absolute top-2.5 right-2.5 flex items-center gap-1.5">
                  <Badge
                    variant="outline"
                    className={cn(
                      "border backdrop-blur-md text-[10px] font-semibold uppercase",
                      c.kind === "html"
                        ? "border-emerald-500/40 bg-zinc-950/80 text-emerald-300"
                        : "border-sky-500/40 bg-zinc-950/80 text-sky-300"
                    )}
                  >
                    {c.kind === "html" ? "Live" : "TSX"}
                  </Badge>
                </div>
              </div>

              <div className="flex flex-1 flex-col justify-between gap-2.5 p-3.5">
                <div className="space-y-1">
                  <h3 className="line-clamp-1 text-sm font-semibold tracking-tight text-foreground group-hover:text-primary">
                    {c.name}
                  </h3>
                  <p className="line-clamp-2 text-xs leading-relaxed text-muted-foreground">
                    {c.description || "UI component"}
                  </p>
                </div>
                <div className="flex items-center justify-between border-t border-border/50 pt-2 text-[11px]">
                  <Badge variant="secondary" className="px-1.5 py-0 text-[10px] font-normal">
                    {CATEGORY_LABELS[c.category] ?? c.category}
                  </Badge>
                  <span className="text-muted-foreground">{c.license.split(" ")[0]}</span>
                </div>
              </div>
            </Card>
          ))}
        </div>
      )}

      <Dialog open={dialogOpen} onOpenChange={setDialogOpen}>
        <DialogContent className="flex h-[88vh] w-[95vw] max-w-5xl flex-col gap-0 overflow-hidden p-0 sm:max-w-5xl">
          <DialogHeader className="border-b border-border/60 bg-card/80 px-6 py-3.5">
            <div className="flex flex-wrap items-center justify-between gap-3 pr-6">
              <div className="space-y-0.5">
                <div className="flex items-center gap-2">
                  <DialogTitle className="text-base font-bold">
                    {selected?.name ?? "Component"}
                  </DialogTitle>
                  {selected ? (
                    <Badge variant="secondary" className="font-mono text-[11px]">
                      {selected.id}
                    </Badge>
                  ) : null}
                </div>
                <DialogDescription className="text-xs">
                  {selected?.description || "UI component"} · {selected?.license}
                </DialogDescription>
              </div>

              <div className="flex flex-wrap items-center gap-2">
                {selected?.kind === "html" ? (
                  <div className="flex items-center rounded-lg border border-border/70 bg-muted/40 p-0.5">
                    <Button
                      variant={dialogTab === "preview" ? "default" : "ghost"}
                      size="xs"
                      onClick={() => setDialogTab("preview")}
                    >
                      <Eye className="size-3.5" />
                      Live
                    </Button>
                    <Button
                      variant={dialogTab === "code" ? "default" : "ghost"}
                      size="xs"
                      onClick={() => setDialogTab("code")}
                    >
                      <FileCode2 className="size-3.5" />
                      Code
                    </Button>
                  </div>
                ) : null}

                <Button variant="outline" size="sm" onClick={handleCopySource}>
                  {copied ? (
                    <Check className="size-3.5 text-emerald-400" />
                  ) : (
                    <Copy className="size-3.5" />
                  )}
                  {copied ? "Copied" : "Copy Source"}
                </Button>

                {selected?.source_url ? (
                  <Button variant="outline" size="sm" asChild>
                    <a href={selected.source_url} target="_blank" rel="noopener noreferrer">
                      <ExternalLink className="size-3.5" />
                      Upstream
                    </a>
                  </Button>
                ) : null}
              </div>
            </div>
          </DialogHeader>

          <div className="flex-1 overflow-hidden bg-muted/20">
            {dialogTab === "preview" && selected?.has_html ? (
              <iframe
                src={selected.html_url ?? undefined}
                title={selected.name}
                className="h-full w-full border-0"
                sandbox="allow-scripts allow-same-origin"
              />
            ) : (
              <div className="h-full overflow-y-auto p-5">
                {detailQuery.isLoading ? (
                  <div className="space-y-3">
                    <Skeleton className="h-6 w-48" />
                    <Skeleton className="h-64 w-full rounded-xl" />
                  </div>
                ) : (
                  <>
                    {selected?.has_preview && selected.preview_url ? (
                      <img
                        src={selected.preview_url}
                        alt={selected.name}
                        className="mb-4 w-full rounded-xl border border-border/70"
                      />
                    ) : null}
                    <pre className="overflow-x-auto rounded-xl border border-border/70 bg-card p-5 font-mono text-xs leading-relaxed whitespace-pre-wrap text-foreground">
                      {sourceText || "No source files on disk for this component."}
                    </pre>
                  </>
                )}
              </div>
            )}
          </div>
        </DialogContent>
      </Dialog>
    </main>
  )
}
