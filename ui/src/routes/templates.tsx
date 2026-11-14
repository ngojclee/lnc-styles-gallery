import { useEffect, useState } from "react"
import { createFileRoute } from "@tanstack/react-router"
import { useQuery } from "@tanstack/react-query"
import {
  Check,
  Copy,
  ExternalLink,
  Eye,
  FileCode2,
  Globe,
  Search,
  Sparkles,
  Terminal,
  X,
} from "lucide-react"
import { toast } from "sonner"

import { fetchTemplateDetail, fetchTemplates } from "@/api/styles"
import { EmptyState } from "@/components/common/EmptyState"
import { PageHeader } from "@/components/common/PageHeader"
import { StatCard } from "@/components/common/StatCard"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { Card } from "@/components/ui/card"
import { copyText } from "@/lib/clipboard"
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog"
import { Input } from "@/components/ui/input"
import { Skeleton } from "@/components/ui/skeleton"
import { cn } from "@/lib/utils"
import type { TemplateItem } from "@/types/styles"

export const Route = createFileRoute("/templates")({
  component: TemplatesPage,
})

function TemplatesPage() {
  const [langFilter, setLangFilter] = useState<string>("all")
  const [searchInput, setSearchInput] = useState("")
  const [debouncedQuery, setDebouncedQuery] = useState("")
  const [selectedTemplate, setSelectedTemplate] = useState<TemplateItem | null>(null)
  const [dialogOpen, setDialogOpen] = useState(false)
  const [dialogTab, setDialogTab] = useState<"preview" | "prompt">("preview")
  const [copiedPrompt, setCopiedPrompt] = useState(false)

  useEffect(() => {
    const timer = setTimeout(() => setDebouncedQuery(searchInput.trim()), 220)
    return () => clearTimeout(timer)
  }, [searchInput])

  const templatesQuery = useQuery({
    queryKey: [
      "templates",
      { lang: langFilter === "all" ? undefined : langFilter, q: debouncedQuery },
    ],
    queryFn: () =>
      fetchTemplates({
        lang: langFilter === "all" ? undefined : langFilter,
        q: debouncedQuery || undefined,
        limit: 100,
      }),
  })

  const detailQuery = useQuery({
    queryKey: ["template-detail", selectedTemplate?.id],
    queryFn: () => fetchTemplateDetail(selectedTemplate!.id),
    enabled: Boolean(dialogOpen && selectedTemplate?.id),
  })

  const items = templatesQuery.data?.items ?? []
  const langFacets = templatesQuery.data?.facets?.languages ?? { en: 50, vi: 50 }

  const handleOpenTemplate = (tpl: TemplateItem, tab: "preview" | "prompt" = "preview") => {
    setSelectedTemplate(tpl)
    setDialogTab(tab)
    setCopiedPrompt(false)
    setDialogOpen(true)
  }

  const handleCopyPrompt = async () => {
    const promptText = detailQuery.data?.prompt || selectedTemplate?.prompt_excerpt || ""
    if (!promptText) {
      toast.error("No prompt available to copy")
      return
    }
    const ok = await copyText(promptText)
    if (ok) {
      setCopiedPrompt(true)
      toast.success(`Copied generation prompt for ${selectedTemplate?.id}`)
      setTimeout(() => setCopiedPrompt(false), 2000)
    } else {
      toast.error("Clipboard copy failed")
    }
  }

  return (
    <main className="flex-1 space-y-6">
      <PageHeader
        icon={FileCode2}
        title="Prompt-Driven HTML Templates"
        description="100 single-file HTML/CSS/JS sample web designs (50 English + 50 Vietnamese) with live iframe previews and generation prompts."
      />

      <div className="grid gap-4 sm:grid-cols-3">
        <StatCard
          label="Total HTML Templates"
          value={ (langFacets.en ?? 50) + (langFacets.vi ?? 50) }
          hint="Self-contained single-file HTML designs"
          icon={FileCode2}
        />
        <StatCard
          label="English Edition (EN)"
          value={langFacets.en ?? 50}
          hint="websites-en/ + prompts-en/"
          icon={Globe}
        />
        <StatCard
          label="Vietnamese Edition (VI)"
          value={langFacets.vi ?? 50}
          hint="websites/ + prompts/"
          icon={Sparkles}
        />
      </div>

      {/* Filter Bar */}
      <Card className="flex flex-col gap-3 p-4 sm:flex-row sm:items-center sm:justify-between">
        <div className="relative flex-1">
          <Search className="pointer-events-none absolute top-1/2 left-3 size-4 -translate-y-1/2 text-muted-foreground" />
          <Input
            value={searchInput}
            onChange={(e) => setSearchInput(e.target.value)}
            placeholder="Search templates by title, industry, or prompt keywords..."
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
          {(["all", "en", "vi"] as const).map((lang) => (
            <Button
              key={lang}
              variant={langFilter === lang ? "default" : "outline"}
              size="sm"
              onClick={() => setLangFilter(lang)}
            >
              {lang === "all"
                ? "All Languages"
                : lang === "en"
                  ? `English (${langFacets.en ?? 50})`
                  : `Tiếng Việt (${langFacets.vi ?? 50})`}
            </Button>
          ))}
        </div>
      </Card>

      {/* Template Grid */}
      {templatesQuery.isLoading ? (
        <div className="grid gap-5 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
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
          icon={FileCode2}
          title="No matching templates"
          description="Try clearing your search input or switching the language filter."
        />
      ) : (
        <div className="grid gap-5 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
          {items.map((tpl) => (
            <Card
              key={tpl.id}
              onClick={() => handleOpenTemplate(tpl, "preview")}
              className="group cursor-pointer gap-0 overflow-hidden p-0 transition-all hover:-translate-y-0.5 hover:border-primary/50 hover:shadow-lg"
            >
              <div className="relative aspect-[16/10] w-full overflow-hidden border-b border-border/60 bg-muted/30">
                <img
                  src={tpl.thumb_url}
                  alt={tpl.title}
                  loading="lazy"
                  className="h-full w-full object-cover transition-transform duration-300 group-hover:scale-[1.03]"
                />
                <div className="absolute top-2.5 right-2.5 flex items-center gap-1.5">
                  <Badge
                    variant="outline"
                    className={cn(
                      "border backdrop-blur-md text-[10px] font-semibold uppercase",
                      tpl.lang === "en"
                        ? "border-sky-500/40 bg-zinc-950/80 text-sky-300"
                        : "border-emerald-500/40 bg-zinc-950/80 text-emerald-300"
                    )}
                  >
                    {tpl.lang.toUpperCase()} #{String(tpl.num).padStart(2, "0")}
                  </Badge>
                </div>
              </div>

              <div className="flex flex-1 flex-col justify-between gap-3 p-4">
                <div className="space-y-1.5">
                  <h3 className="line-clamp-1 font-semibold tracking-tight text-foreground group-hover:text-primary">
                    {tpl.title}
                  </h3>
                  <p className="line-clamp-2 text-xs leading-relaxed text-muted-foreground">
                    {tpl.prompt_excerpt || "Prompt-driven single-file HTML/CSS/JS template"}
                  </p>
                </div>

                <div className="flex items-center justify-between border-t border-border/50 pt-2.5 text-xs">
                  <button
                    type="button"
                    onClick={(e) => {
                      e.stopPropagation()
                      handleOpenTemplate(tpl, "prompt")
                    }}
                    className="inline-flex items-center gap-1 text-muted-foreground hover:text-foreground"
                  >
                    <Terminal className="size-3" />
                    Prompt
                  </button>
                  <span className="inline-flex items-center gap-1 font-medium text-primary group-hover:underline">
                    <Eye className="size-3.5" />
                    Live Preview
                  </span>
                </div>
              </div>
            </Card>
          ))}
        </div>
      )}

      {/* Template Iframe + Prompt Modal */}
      <Dialog open={dialogOpen} onOpenChange={setDialogOpen}>
        <DialogContent className="flex h-[88vh] w-[95vw] max-w-6xl flex-col gap-0 overflow-hidden p-0 sm:max-w-6xl">
          <DialogHeader className="border-b border-border/60 bg-card/80 px-6 py-3.5">
            <div className="flex flex-wrap items-center justify-between gap-3 pr-6">
              <div className="space-y-0.5">
                <div className="flex items-center gap-2">
                  <DialogTitle className="text-base font-bold">
                    {selectedTemplate?.title ?? "Template Preview"}
                  </DialogTitle>
                  {selectedTemplate ? (
                    <Badge variant="secondary" className="font-mono text-[11px]">
                      {selectedTemplate.id}
                    </Badge>
                  ) : null}
                </div>
                <DialogDescription className="text-xs">
                  Single-file HTML/CSS/JS sample design from prompt-driven-ui
                </DialogDescription>
              </div>

              <div className="flex flex-wrap items-center gap-2">
                <div className="flex items-center rounded-lg border border-border/70 bg-muted/40 p-0.5">
                  <Button
                    variant={dialogTab === "preview" ? "default" : "ghost"}
                    size="xs"
                    onClick={() => setDialogTab("preview")}
                  >
                    <Eye className="size-3.5" />
                    Live HTML Preview
                  </Button>
                  <Button
                    variant={dialogTab === "prompt" ? "default" : "ghost"}
                    size="xs"
                    onClick={() => setDialogTab("prompt")}
                  >
                    <Terminal className="size-3.5" />
                    Generation Prompt
                  </Button>
                </div>

                <Button variant="outline" size="sm" onClick={handleCopyPrompt}>
                  {copiedPrompt ? (
                    <Check className="size-3.5 text-emerald-400" />
                  ) : (
                    <Copy className="size-3.5" />
                  )}
                  {copiedPrompt ? "Copied" : "Copy Prompt"}
                </Button>

                {selectedTemplate ? (
                  <Button
                    variant="outline"
                    size="sm"
                    asChild
                  >
                    <a
                      href={selectedTemplate.html_url}
                      target="_blank"
                      rel="noopener noreferrer"
                    >
                      <ExternalLink className="size-3.5" />
                      Open Standalone
                    </a>
                  </Button>
                ) : null}
              </div>
            </div>
          </DialogHeader>

          <div className="flex-1 overflow-hidden bg-muted/20">
            {dialogTab === "preview" && selectedTemplate ? (
              <iframe
                src={selectedTemplate.html_url}
                title={selectedTemplate.title}
                className="h-full w-full border-0 bg-white"
                sandbox="allow-scripts allow-same-origin"
              />
            ) : (
              <div className="h-full overflow-y-auto p-6">
                {detailQuery.isLoading ? (
                  <div className="space-y-3">
                    <Skeleton className="h-6 w-48" />
                    <Skeleton className="h-64 w-full rounded-xl" />
                  </div>
                ) : (
                  <pre className="overflow-x-auto rounded-xl border border-border/70 bg-card p-5 font-mono text-xs leading-relaxed whitespace-pre-wrap text-foreground">
                    {detailQuery.data?.prompt ||
                      selectedTemplate?.prompt_excerpt ||
                      "No prompt text available."}
                  </pre>
                )}
              </div>
            )}
          </div>
        </DialogContent>
      </Dialog>
    </main>
  )
}
