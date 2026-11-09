import { useEffect, useMemo, useState } from "react"
import { useQuery } from "@tanstack/react-query"
import {
  Check,
  Copy,
  ExternalLink,
  FileCode2,
  FileText,
  Image as ImageIcon,
  Layers,
  Moon,
  Sparkles,
  Sun,
} from "lucide-react"
import { toast } from "sonner"

import { fetchStyleDetail, fetchStyleShots } from "@/api/styles"
import { ZoomViewport } from "@/components/common/ZoomViewport"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog"
import { Skeleton } from "@/components/ui/skeleton"
import { cn } from "@/lib/utils"

interface StyleDetailDialogProps {
  styleId: string | null
  open: boolean
  onOpenChange: (open: boolean) => void
}

type ViewTab = "rendered" | "raw" | "meta"

function escapeHtml(value: string): string {
  return value
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#39;")
}

function renderInlineMarkdown(text: string): string {
  let out = escapeHtml(text)
  // Inline hex swatches (#RRGGBB or #RGB)
  out = out.replace(/`(#[0-9a-fA-F]{3,8})`/g, (_m, hex: string) => {
    return `<span class="inline-flex items-center gap-1.5 rounded border border-border/70 bg-muted/60 px-1.5 py-0.5 font-mono text-xs"><span class="inline-block size-2.5 rounded-full border border-white/20" style="background:${hex}"></span>${hex}</span>`
  })
  out = out.replace(/(^|\s)(#[0-9a-fA-F]{6})\b/g, (_m, prefix: string, hex: string) => {
    return `${prefix}<span class="inline-flex items-center gap-1.5 rounded border border-border/70 bg-muted/60 px-1.5 py-0.5 font-mono text-xs"><span class="inline-block size-2.5 rounded-full border border-white/20" style="background:${hex}"></span>${hex}</span>`
  })
  // Inline code
  out = out.replace(
    /`([^`]+)`/g,
    '<code class="rounded bg-muted px-1.5 py-0.5 font-mono text-xs text-primary">$1</code>'
  )
  // Bold & italic
  out = out.replace(/\*\*([^*]+)\*\*/g, '<strong class="font-semibold text-foreground">$1</strong>')
  out = out.replace(/\*([^*]+)\*/g, "<em>$1</em>")
  // Links
  out = out.replace(
    /\[([^\]]+)\]\((https?:\/\/[^)]+)\)/g,
    '<a href="$2" target="_blank" rel="noopener noreferrer" class="text-primary underline underline-offset-4 hover:opacity-80">$1</a>'
  )
  return out
}

function renderMarkdownToHtml(md: string): string {
  if (!md) return '<p class="text-muted-foreground">No DESIGN.md content available for this style.</p>'
  const lines = md.replace(/\r\n/g, "\n").split("\n")
  const html: string[] = []
  let inCodeBlock = false
  let codeLines: string[] = []
  let inList: "ul" | "ol" | null = null
  let inTable = false
  let tableRows: string[][] = []

  const flushList = () => {
    if (inList) {
      html.push(`</${inList}>`)
      inList = null
    }
  }

  const flushTable = () => {
    if (!inTable || tableRows.length === 0) return
    inTable = false
    const header = tableRows[0] ?? []
    const body = tableRows.slice(1).filter((row) => !row.every((c) => /^[-:\s]+$/.test(c)))
    html.push(
      '<div class="my-4 overflow-x-auto rounded-lg border border-border/70"><table class="w-full text-left text-xs">'
    )
    html.push('<thead class="border-b border-border/70 bg-muted/50"><tr>')
    for (const cell of header) {
      html.push(`<th class="px-3 py-2 font-semibold text-foreground">${renderInlineMarkdown(cell)}</th>`)
    }
    html.push("</tr></thead><tbody>")
    for (const row of body) {
      html.push('<tr class="border-b border-border/40 last:border-0">')
      for (const cell of row) {
        html.push(`<td class="px-3 py-2 text-muted-foreground">${renderInlineMarkdown(cell)}</td>`)
      }
      html.push("</tr>")
    }
    html.push("</tbody></table></div>")
    tableRows = []
  }

  for (const rawLine of lines) {
    if (rawLine.trim().startsWith("```")) {
      flushList()
      flushTable()
      if (!inCodeBlock) {
        inCodeBlock = true
        codeLines = []
      } else {
        inCodeBlock = false
        html.push(
          `<pre class="my-3 overflow-x-auto rounded-lg border border-border/70 bg-muted/60 p-3.5 font-mono text-xs leading-relaxed text-foreground"><code>${escapeHtml(
            codeLines.join("\n")
          )}</code></pre>`
        )
      }
      continue
    }

    if (inCodeBlock) {
      codeLines.push(rawLine)
      continue
    }

    const trimmed = rawLine.trim()
    if (!trimmed) {
      flushList()
      flushTable()
      continue
    }

    // Table row
    if (trimmed.startsWith("|") && trimmed.endsWith("|")) {
      flushList()
      inTable = true
      const cells = trimmed
        .slice(1, -1)
        .split("|")
        .map((c) => c.trim())
      tableRows.push(cells)
      continue
    } else if (inTable) {
      flushTable()
    }

    if (/^---+$/.test(trimmed) || /^\*\*\*+$/.test(trimmed)) {
      flushList()
      html.push('<hr class="my-5 border-border/60" />')
      continue
    }

    const hMatch = /^(#{1,4})\s+(.+)$/.exec(trimmed)
    if (hMatch) {
      flushList()
      const level = hMatch[1]?.length ?? 2
      const content = renderInlineMarkdown(hMatch[2] ?? "")
      if (level === 1) {
        html.push(`<h1 class="mt-6 mb-3 text-xl font-bold tracking-tight text-foreground first:mt-0">${content}</h1>`)
      } else if (level === 2) {
        html.push(
          `<h2 class="mt-6 mb-2.5 border-b border-border/60 pb-1.5 text-base font-semibold tracking-tight text-foreground">${content}</h2>`
        )
      } else if (level === 3) {
        html.push(`<h3 class="mt-4 mb-1.5 text-sm font-semibold text-foreground">${content}</h3>`)
      } else {
        html.push(`<h4 class="mt-3 mb-1 text-xs font-semibold uppercase tracking-wider text-muted-foreground">${content}</h4>`)
      }
      continue
    }

    if (trimmed.startsWith("> ")) {
      flushList()
      html.push(
        `<blockquote class="my-3 border-l-2 border-primary/60 bg-muted/30 py-1.5 pl-3.5 italic text-muted-foreground">${renderInlineMarkdown(
          trimmed.slice(2)
        )}</blockquote>`
      )
      continue
    }

    const ulMatch = /^[-*+]\s+(.+)$/.exec(trimmed)
    if (ulMatch) {
      if (inList !== "ul") {
        flushList()
        inList = "ul"
        html.push('<ul class="my-2.5 list-disc space-y-1.5 pl-5 text-sm text-muted-foreground">')
      }
      html.push(`<li class="leading-relaxed">${renderInlineMarkdown(ulMatch[1] ?? "")}</li>`)
      continue
    }

    const olMatch = /^\d+\.\s+(.+)$/.exec(trimmed)
    if (olMatch) {
      if (inList !== "ol") {
        flushList()
        inList = "ol"
        html.push('<ol class="my-2.5 list-decimal space-y-1.5 pl-5 text-sm text-muted-foreground">')
      }
      html.push(`<li class="leading-relaxed">${renderInlineMarkdown(olMatch[1] ?? "")}</li>`)
      continue
    }

    flushList()
    html.push(`<p class="my-2.5 text-sm leading-relaxed text-muted-foreground">${renderInlineMarkdown(trimmed)}</p>`)
  }

  flushList()
  flushTable()
  return html.join("\n")
}

export function StyleDetailDialog({ styleId, open, onOpenChange }: StyleDetailDialogProps) {
  const [viewTab, setViewTab] = useState<ViewTab>("rendered")
  const [selectedShot, setSelectedShot] = useState<{ url: string; kind: string } | null>(null)
  const [copied, setCopied] = useState(false)

  const detailQuery = useQuery({
    queryKey: ["style-detail", styleId],
    queryFn: () => fetchStyleDetail(styleId!),
    enabled: Boolean(open && styleId),
  })

  const shotsQuery = useQuery({
    queryKey: ["style-shots", styleId],
    queryFn: () => fetchStyleShots(styleId!),
    enabled: Boolean(open && styleId),
  })

  const style = detailQuery.data
  const shots = useMemo(() => {
    if (shotsQuery.data && shotsQuery.data.length > 0) return shotsQuery.data
    return style?.shots ?? []
  }, [shotsQuery.data, style?.shots])

  const defaultHeroUrl = styleId ? `/api/styles/${encodeURIComponent(styleId)}/preview` : null
  const activeImageUrl = selectedShot?.url ?? defaultHeroUrl
  const activeIsVideo = selectedShot?.kind === "video"

  useEffect(() => {
    setSelectedShot(null)
    setViewTab("rendered")
    setCopied(false)
  }, [styleId])

  const renderedHtml = useMemo(() => {
    return renderMarkdownToHtml(style?.design_md ?? "")
  }, [style?.design_md])

  const handleCopyMarkdown = async () => {
    if (!style?.design_md) {
      toast.error("No DESIGN.md content to copy")
      return
    }
    try {
      await navigator.clipboard.writeText(style.design_md)
      setCopied(true)
      toast.success(`Copied DESIGN.md for "${style.title}"`)
      setTimeout(() => setCopied(false), 2000)
    } catch {
      toast.error("Clipboard copy failed")
    }
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="flex max-h-[90vh] w-[95vw] max-w-6xl flex-col gap-0 overflow-hidden p-0 sm:max-w-6xl">
        <DialogHeader className="border-b border-border/60 bg-card/80 px-6 py-4">
          <div className="flex flex-wrap items-start justify-between gap-4 pr-6">
            <div className="space-y-1.5">
              <div className="flex flex-wrap items-center gap-2">
                <DialogTitle className="text-lg font-bold tracking-tight">
                  {style?.title ?? "Loading style..."}
                </DialogTitle>
                {style ? (
                  <>
                    <Badge
                      variant="outline"
                      className={cn(
                        "gap-1 text-[11px] uppercase",
                        style.theme === "dark"
                          ? "border-violet-500/40 bg-violet-500/10 text-violet-300"
                          : "border-amber-500/40 bg-amber-500/10 text-amber-300"
                      )}
                    >
                      {style.theme === "dark" ? <Moon className="size-3" /> : <Sun className="size-3" />}
                      {style.theme}
                    </Badge>
                    <Badge variant="secondary" className="text-[11px]">
                      {style.category_label}
                    </Badge>
                  </>
                ) : null}
              </div>
              <DialogDescription className="line-clamp-2 text-xs text-muted-foreground">
                {style?.tagline || "Design system specification & visual reference"}
              </DialogDescription>
            </div>

            {style ? (
              <div className="flex flex-wrap items-center gap-2">
                <Button variant="outline" size="sm" onClick={handleCopyMarkdown}>
                  {copied ? <Check className="size-3.5 text-emerald-400" /> : <Copy className="size-3.5" />}
                  {copied ? "Copied DESIGN.md" : "Copy DESIGN.md"}
                </Button>
                {style.source_url ? (
                  <Button
                    variant="outline"
                    size="sm"
                    asChild
                  >
                    <a href={style.source_url} target="_blank" rel="noopener noreferrer">
                      <ExternalLink className="size-3.5" />
                      Product Site
                    </a>
                  </Button>
                ) : null}
                {style.url ? (
                  <Button
                    variant="ghost"
                    size="sm"
                    asChild
                  >
                    <a href={style.url} target="_blank" rel="noopener noreferrer">
                      <ExternalLink className="size-3.5" />
                      Refero
                    </a>
                  </Button>
                ) : null}
              </div>
            ) : null}
          </div>
        </DialogHeader>

        {detailQuery.isLoading ? (
          <div className="grid flex-1 gap-6 overflow-y-auto p-6 lg:grid-cols-12">
            <div className="space-y-4 lg:col-span-5">
              <Skeleton className="aspect-[16/10] w-full rounded-xl" />
              <div className="grid grid-cols-4 gap-2">
                <Skeleton className="aspect-[16/10] rounded-lg" />
                <Skeleton className="aspect-[16/10] rounded-lg" />
                <Skeleton className="aspect-[16/10] rounded-lg" />
                <Skeleton className="aspect-[16/10] rounded-lg" />
              </div>
            </div>
            <div className="space-y-3 lg:col-span-7">
              <Skeleton className="h-8 w-64" />
              <Skeleton className="h-4 w-full" />
              <Skeleton className="h-4 w-5/6" />
              <Skeleton className="h-64 w-full rounded-xl" />
            </div>
          </div>
        ) : style ? (
          <div className="grid flex-1 overflow-y-auto lg:grid-cols-12 lg:overflow-hidden">
            {/* Left column: Hero preview + shots strip */}
            <div className="flex flex-col gap-4 border-b border-border/60 bg-muted/15 p-5 lg:col-span-5 lg:overflow-y-auto lg:border-r lg:border-b-0">
              <div className="space-y-2">
                <div className="flex items-center justify-between text-xs text-muted-foreground">
                  <span className="flex items-center gap-1.5 font-medium text-foreground">
                    <ImageIcon className="size-3.5 text-primary" />
                    Visual Preview
                  </span>
                  <span>Click image to zoom</span>
                </div>
                {activeImageUrl ? (
                  activeIsVideo ? (
                    <div className="h-[320px] sm:h-[400px]">
                      <video
                        key={activeImageUrl}
                        src={activeImageUrl}
                        controls
                        muted
                        loop
                        playsInline
                        className="h-full w-full rounded-lg border border-border/70 bg-black object-contain shadow-sm"
                      />
                    </div>
                  ) : (
                    <div className="h-[320px] sm:h-[400px]">
                      <ZoomViewport
                        imageUrl={activeImageUrl}
                        alt={style.title}
                        contentWidth={1280}
                        contentHeight={800}
                        className="border border-border/70 shadow-sm"
                      />
                    </div>
                  )
                ) : null}
              </div>

              {/* Image Strip: Hero + Shots */}
              <div className="space-y-2">
                <div className="flex items-center justify-between text-xs">
                  <span className="font-medium text-foreground">
                    Gallery Shots ({shots.length > 0 ? `${shots.length} shots` : "Hero preview"})
                  </span>
                  {selectedShot ? (
                    <button
                      type="button"
                      onClick={() => setSelectedShot(null)}
                      className="text-xs text-primary hover:underline"
                    >
                      Reset to Hero
                    </button>
                  ) : null}
                </div>
                <div className="grid grid-cols-4 gap-2">
                  {defaultHeroUrl ? (
                    <button
                      type="button"
                      onClick={() => setSelectedShot(null)}
                      className={cn(
                        "group relative aspect-[16/10] overflow-hidden rounded-lg border transition-all",
                        !selectedShot
                          ? "border-primary ring-2 ring-primary/30"
                          : "border-border/60 opacity-75 hover:opacity-100"
                      )}
                    >
                      <img
                        src={defaultHeroUrl}
                        alt="Hero preview"
                        className="h-full w-full object-cover"
                        loading="lazy"
                      />
                      <span className="absolute bottom-1 left-1 rounded bg-background/85 px-1.5 py-0.5 text-[10px] font-medium">
                        Hero
                      </span>
                    </button>
                  ) : null}
                  {shots.map((shot) => {
                    const isSelected = selectedShot?.url === shot.url
                    return (
                      <button
                        key={shot.n}
                        type="button"
                        onClick={() => setSelectedShot({ url: shot.url, kind: shot.kind ?? "image" })}
                        className={cn(
                          "group relative aspect-[16/10] overflow-hidden rounded-lg border transition-all",
                          isSelected
                            ? "border-primary ring-2 ring-primary/30"
                            : "border-border/60 opacity-75 hover:opacity-100"
                        )}
                      >
                        {shot.kind === "video" ? (
                          <video
                            src={shot.url}
                            muted
                            playsInline
                            preload="metadata"
                            className="h-full w-full object-cover"
                          />
                        ) : (
                          <img
                            src={shot.url}
                            alt={`${style.title} shot #${shot.n}`}
                            className="h-full w-full object-cover"
                            loading="lazy"
                          />
                        )}
                        <span className="absolute bottom-1 left-1 rounded bg-background/85 px-1.5 py-0.5 text-[10px] font-mono">
                          {shot.kind === "video" ? "▶" : `#${shot.n}`}
                        </span>
                      </button>
                    )
                  })}
                </div>
              </div>

              {/* Quick Metadata Summary */}
              <div className="mt-auto rounded-xl border border-border/60 bg-card/60 p-3.5 text-xs">
                <div className="mb-2 flex items-center gap-1.5 font-semibold text-foreground">
                  <Sparkles className="size-3.5 text-primary" />
                  Style Attributes
                </div>
                <dl className="grid grid-cols-2 gap-y-1.5 text-muted-foreground">
                  <dt>Style UUID</dt>
                  <dd className="truncate font-mono text-foreground" title={style.id}>
                    {style.id.slice(0, 13)}…
                  </dd>
                  <dt>Category</dt>
                  <dd className="text-foreground">{style.category_label}</dd>
                  <dt>Color Mode</dt>
                  <dd className="capitalize text-foreground">{style.theme}</dd>
                  <dt>DESIGN.md</dt>
                  <dd className="text-foreground">
                    {style.design_md ? `${(style.design_md.length / 1024).toFixed(1)} KB` : "Missing"}
                  </dd>
                </dl>
              </div>
            </div>

            {/* Right column: DESIGN.md viewer (Rendered | Raw | Meta) */}
            <div className="flex flex-col lg:col-span-7 lg:overflow-hidden">
              <div className="flex items-center justify-between border-b border-border/60 bg-muted/20 px-5 py-2.5">
                <div className="flex items-center gap-1">
                  <Button
                    variant={viewTab === "rendered" ? "default" : "ghost"}
                    size="xs"
                    onClick={() => setViewTab("rendered")}
                  >
                    <FileText className="size-3.5" />
                    Rendered
                  </Button>
                  <Button
                    variant={viewTab === "raw" ? "default" : "ghost"}
                    size="xs"
                    onClick={() => setViewTab("raw")}
                  >
                    <FileCode2 className="size-3.5" />
                    Raw
                  </Button>
                  <Button
                    variant={viewTab === "meta" ? "default" : "ghost"}
                    size="xs"
                    onClick={() => setViewTab("meta")}
                  >
                    <Layers className="size-3.5" />
                    Meta
                  </Button>
                </div>

                <Button variant="ghost" size="xs" onClick={handleCopyMarkdown}>
                  <Copy className="size-3" />
                  Copy
                </Button>
              </div>

              <div className="flex-1 overflow-y-auto p-6">
                {viewTab === "rendered" ? (
                  <div
                    className="max-w-none space-y-2"
                    dangerouslySetInnerHTML={{ __html: renderedHtml }}
                  />
                ) : null}

                {viewTab === "raw" ? (
                  <pre className="overflow-x-auto rounded-xl border border-border/70 bg-muted/50 p-4 font-mono text-xs leading-relaxed text-foreground">
                    {style.design_md || "// No DESIGN.md file present for this style."}
                  </pre>
                ) : null}

                {viewTab === "meta" ? (
                  <pre className="overflow-x-auto rounded-xl border border-border/70 bg-muted/50 p-4 font-mono text-xs leading-relaxed text-foreground">
                    {JSON.stringify(
                      {
                        id: style.id,
                        title: style.title,
                        tagline: style.tagline,
                        theme: style.theme,
                        category: style.category,
                        category_label: style.category_label,
                        url: style.url,
                        source_url: style.source_url,
                        has_design: style.has_design,
                        has_preview: style.has_preview,
                        preview_url: style.preview_url,
                        shots_count: shots.length,
                        meta: style.meta,
                      },
                      null,
                      2
                    )}
                  </pre>
                ) : null}
              </div>
            </div>
          </div>
        ) : (
          <div className="p-8 text-center text-sm text-muted-foreground">
            Unable to load style details.
          </div>
        )}
      </DialogContent>
    </Dialog>
  )
}
