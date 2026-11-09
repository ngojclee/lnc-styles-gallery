import { useEffect, useMemo, useState } from "react"
import { createFileRoute } from "@tanstack/react-router"
import { useQuery } from "@tanstack/react-query"
import {
  ChevronLeft,
  ChevronRight,
  ExternalLink,
  FileText,
  Filter,
  FolderKanban,
  Image as ImageIcon,
  Moon,
  Palette,
  RotateCcw,
  Search,
  Sparkles,
  Sun,
  X,
} from "lucide-react"

import {
  fetchCollectionDetail,
  fetchSearchStyles,
  fetchStyles,
  healthQuery,
} from "@/api/styles"
import { EmptyState } from "@/components/common/EmptyState"
import { PageHeader } from "@/components/common/PageHeader"
import { StatCard } from "@/components/common/StatCard"
import { StyleDetailDialog } from "@/components/common/StyleDetailDialog"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { Card } from "@/components/ui/card"
import { Input } from "@/components/ui/input"
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select"
import { Skeleton } from "@/components/ui/skeleton"
import { Switch } from "@/components/ui/switch"
import { cn } from "@/lib/utils"
import type { StyleItem } from "@/types/styles"

export interface PendingStyleFilter {
  category?: string
  theme?: string
  collectionId?: string
  collectionName?: string
}

let pendingStyleFilter: PendingStyleFilter | null = null

export function setPendingStyleFilter(filter: PendingStyleFilter | null) {
  pendingStyleFilter = filter
}

export const Route = createFileRoute("/styles")({
  component: StylesGalleryPage,
})

const PAGE_SIZE = 48

export function StylesGalleryPage() {
  const { data: health } = useQuery(healthQuery)

  const [searchInput, setSearchInput] = useState("")
  const [debouncedQuery, setDebouncedQuery] = useState("")
  const [searchDesignBody, setSearchDesignBody] = useState(true)
  const [themeFilter, setThemeFilter] = useState<string>("all")
  const [categoryFilter, setCategoryFilter] = useState<string>("all")
  const [collectionFilter, setCollectionFilter] = useState<{
    id: string
    name: string
  } | null>(null)
  const [page, setPage] = useState(0)

  const [selectedStyleId, setSelectedStyleId] = useState<string | null>(null)
  const [detailOpen, setDetailOpen] = useState(false)

  // Apply pending filter if navigated from Collections page
  useEffect(() => {
    if (pendingStyleFilter) {
      if (pendingStyleFilter.category) {
        setCategoryFilter(pendingStyleFilter.category)
      } else {
        setCategoryFilter("all")
      }
      if (pendingStyleFilter.theme) {
        setThemeFilter(pendingStyleFilter.theme)
      } else {
        setThemeFilter("all")
      }
      if (pendingStyleFilter.collectionId) {
        setCollectionFilter({
          id: pendingStyleFilter.collectionId,
          name: pendingStyleFilter.collectionName ?? pendingStyleFilter.collectionId,
        })
      } else {
        setCollectionFilter(null)
      }
      setPage(0)
      pendingStyleFilter = null
    }
  }, [])

  // Debounce search query
  useEffect(() => {
    const timer = setTimeout(() => {
      setDebouncedQuery(searchInput.trim())
      setPage(0)
    }, 240)
    return () => clearTimeout(timer)
  }, [searchInput])

  const activeTheme = themeFilter === "all" ? undefined : themeFilter
  const activeCategory = categoryFilter === "all" ? undefined : categoryFilter

  // Standard catalog query
  const listQuery = useQuery({
    queryKey: [
      "styles-list",
      {
        q: debouncedQuery,
        theme: activeTheme,
        category: activeCategory,
        collection: collectionFilter?.id,
        page,
      },
    ],
    queryFn: () =>
      fetchStyles({
        q: debouncedQuery || undefined,
        theme: activeTheme,
        category: activeCategory,
        collection: collectionFilter?.id,
        limit: PAGE_SIZE,
        offset: page * PAGE_SIZE,
      }),
    enabled: !debouncedQuery || !searchDesignBody,
  })

  // Full-text DESIGN.md search query when query is non-empty and searchDesignBody is enabled
  const fullTextQuery = useQuery({
    queryKey: [
      "styles-search",
      {
        q: debouncedQuery,
        theme: activeTheme,
        category: activeCategory,
        include_body: searchDesignBody,
      },
    ],
    queryFn: () =>
      fetchSearchStyles({
        q: debouncedQuery,
        theme: activeTheme,
        category: activeCategory,
        include_body: searchDesignBody,
        limit: 96,
      }),
    enabled: Boolean(debouncedQuery && searchDesignBody),
  })

  // Optional collection detail if a sitemap collection is active
  const collectionQuery = useQuery({
    queryKey: ["collection-detail", collectionFilter?.id, page],
    queryFn: () =>
      fetchCollectionDetail(collectionFilter!.id, PAGE_SIZE, page * PAGE_SIZE),
    enabled: Boolean(collectionFilter?.id && !debouncedQuery),
  })

  const categories = useMemo(() => {
    return listQuery.data?.facets?.categories ?? health?.categories ?? []
  }, [listQuery.data?.facets?.categories, health?.categories])

  const { items, totalCount, isLoading } = useMemo(() => {
    if (debouncedQuery && searchDesignBody) {
      return {
        items: fullTextQuery.data?.styles ?? [],
        totalCount: fullTextQuery.data?.total_styles ?? 0,
        isLoading: fullTextQuery.isLoading,
      }
    }
    if (collectionFilter?.id && !debouncedQuery && collectionQuery.data) {
      return {
        items: collectionQuery.data.styles ?? [],
        totalCount: collectionQuery.data.count ?? 0,
        isLoading: collectionQuery.isLoading,
      }
    }
    return {
      items: listQuery.data?.items ?? [],
      totalCount: listQuery.data?.total ?? 0,
      isLoading: listQuery.isLoading,
    }
  }, [
    debouncedQuery,
    searchDesignBody,
    fullTextQuery.data,
    fullTextQuery.isLoading,
    collectionFilter,
    collectionQuery.data,
    collectionQuery.isLoading,
    listQuery.data,
    listQuery.isLoading,
  ])

  const totalPages = Math.max(1, Math.ceil(totalCount / PAGE_SIZE))

  const handleOpenStyle = (style: StyleItem) => {
    setSelectedStyleId(style.id)
    setDetailOpen(true)
  }

  const handleResetFilters = () => {
    setSearchInput("")
    setDebouncedQuery("")
    setThemeFilter("all")
    setCategoryFilter("all")
    setCollectionFilter(null)
    setPage(0)
  }

  const hasActiveFilters =
    Boolean(searchInput.trim()) ||
    themeFilter !== "all" ||
    categoryFilter !== "all" ||
    Boolean(collectionFilter)

  return (
    <main className="flex-1 space-y-6 p-6">
      <PageHeader
        icon={Palette}
        title="Design Styles Catalog"
        description="Curated real-world product design systems with visual previews, screenshots, and full DESIGN.md specifications."
        actions={
          hasActiveFilters ? (
            <Button variant="outline" size="sm" onClick={handleResetFilters}>
              <RotateCcw className="size-3.5" />
              Reset Filters
            </Button>
          ) : null
        }
      />

      {/* Corpus Stat Cards */}
      <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
        <StatCard
          label="Curated Design Systems"
          value={health?.styles_count ?? 1236}
          hint={`${health?.previews_count ?? 0} hero previews indexed`}
          icon={Palette}
        />
        <StatCard
          label="DESIGN.md Specifications"
          value={health?.designs_count ?? 1236}
          hint="Color tokens, typography, layout & rules"
          icon={FileText}
        />
        <StatCard
          label="Style Collections"
          value={health?.collections_count ?? 15}
          hint="Grouped by industry, mood & theme"
          icon={FolderKanban}
        />
        <StatCard
          label="Theme Distribution"
          value={`${health?.themes?.light ?? 0}L / ${health?.themes?.dark ?? 0}D`}
          hint="Light vs Dark product UI systems"
          icon={Sparkles}
        />
      </div>

      {/* Search & Filter Toolbar */}
      <Card className="gap-4 p-4">
        <div className="flex flex-col gap-3 lg:flex-row lg:items-center">
          <div className="relative flex-1">
            <Search className="pointer-events-none absolute top-1/2 left-3 size-4 -translate-y-1/2 text-muted-foreground" />
            <Input
              value={searchInput}
              onChange={(e) => setSearchInput(e.target.value)}
              placeholder="Search styles by product name, tagline, font family, hex token, or layout keyword..."
              className="pl-9 pr-8"
            />
            {searchInput ? (
              <button
                type="button"
                onClick={() => setSearchInput("")}
                className="absolute top-1/2 right-2.5 -translate-y-1/2 text-muted-foreground hover:text-foreground"
                aria-label="Clear search"
              >
                <X className="size-3.5" />
              </button>
            ) : null}
          </div>

          <div className="flex flex-wrap items-center gap-2.5">
            {/* Theme Filter */}
            <Select
              value={themeFilter}
              onValueChange={(val) => {
                setThemeFilter(val ?? "all")
                setPage(0)
              }}
            >
              <SelectTrigger className="w-[140px]">
                <SelectValue placeholder="All Themes" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="all">All Themes</SelectItem>
                <SelectItem value="light">Light Theme</SelectItem>
                <SelectItem value="dark">Dark Theme</SelectItem>
              </SelectContent>
            </Select>

            {/* Category Filter */}
            <Select
              value={categoryFilter}
              onValueChange={(val) => {
                setCategoryFilter(val ?? "all")
                setCollectionFilter(null)
                setPage(0)
              }}
            >
              <SelectTrigger className="w-[200px]">
                <SelectValue placeholder="All Categories" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="all">All Categories</SelectItem>
                {categories.map((cat) => (
                  <SelectItem key={cat.id} value={cat.id}>
                    {cat.label} ({cat.count})
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>

            {/* Deep search toggle */}
            <label className="flex cursor-pointer items-center gap-2 rounded-lg border border-border/70 bg-muted/30 px-3 py-1.5 text-xs text-muted-foreground select-none">
              <Switch
                size="sm"
                checked={searchDesignBody}
                onCheckedChange={(checked) => setSearchDesignBody(Boolean(checked))}
              />
              <span>Search DESIGN.md body</span>
            </label>
          </div>
        </div>

        {/* Category Quick Pills */}
        {categories.length > 0 ? (
          <div className="flex flex-wrap items-center gap-1.5 border-t border-border/50 pt-3">
            <span className="mr-1 inline-flex items-center gap-1 text-[11px] font-medium text-muted-foreground">
              <Filter className="size-3" />
              Categories:
            </span>
            <button
              type="button"
              onClick={() => {
                setCategoryFilter("all")
                setCollectionFilter(null)
                setPage(0)
              }}
              className={cn(
                "rounded-full px-2.5 py-1 text-xs font-medium transition-colors",
                categoryFilter === "all" && !collectionFilter
                  ? "bg-primary text-primary-foreground"
                  : "bg-muted/60 text-muted-foreground hover:bg-muted hover:text-foreground"
              )}
            >
              All
            </button>
            {categories.map((cat) => {
              const active = categoryFilter === cat.id
              return (
                <button
                  key={cat.id}
                  type="button"
                  onClick={() => {
                    setCategoryFilter(active ? "all" : cat.id)
                    setCollectionFilter(null)
                    setPage(0)
                  }}
                  className={cn(
                    "rounded-full px-2.5 py-1 text-xs font-medium transition-colors",
                    active
                      ? "bg-primary text-primary-foreground"
                      : "bg-muted/60 text-muted-foreground hover:bg-muted hover:text-foreground"
                  )}
                >
                  {cat.label}{" "}
                  <span className="ml-0.5 font-mono text-[10px] opacity-75">
                    {cat.count}
                  </span>
                </button>
              )
            })}
          </div>
        ) : null}

        {/* Active Collection Banner */}
        {collectionFilter ? (
          <div className="flex items-center justify-between rounded-lg border border-primary/30 bg-primary/10 px-3 py-2 text-xs">
            <span className="font-medium text-foreground">
              Filtered by Collection: <strong>{collectionFilter.name}</strong>
            </span>
            <Button
              variant="ghost"
              size="xs"
              onClick={() => setCollectionFilter(null)}
            >
              <X className="size-3" />
              Clear Collection
            </Button>
          </div>
        ) : null}
      </Card>

      {/* Result Count & Pagination Bar */}
      <div className="flex flex-wrap items-center justify-between gap-2 text-xs text-muted-foreground">
        <div>
          Showing <strong className="text-foreground">{items.length}</strong> of{" "}
          <strong className="text-foreground">{totalCount}</strong> design styles
          {debouncedQuery ? (
            <span>
              {" "}
              matching <code className="text-primary">"{debouncedQuery}"</code>
            </span>
          ) : null}
        </div>

        {!debouncedQuery && totalPages > 1 ? (
          <div className="flex items-center gap-2">
            <Button
              variant="outline"
              size="xs"
              disabled={page <= 0}
              onClick={() => setPage((p) => Math.max(0, p - 1))}
            >
              <ChevronLeft className="size-3.5" />
              Prev
            </Button>
            <span className="font-mono text-xs">
              Page {page + 1} / {totalPages}
            </span>
            <Button
              variant="outline"
              size="xs"
              disabled={page + 1 >= totalPages}
              onClick={() => setPage((p) => p + 1)}
            >
              Next
              <ChevronRight className="size-3.5" />
            </Button>
          </div>
        ) : null}
      </div>

      {/* Style Cards Grid */}
      {isLoading ? (
        <div className="grid gap-5 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
          {Array.from({ length: 12 }).map((_, idx) => (
            <Card key={idx} className="overflow-hidden p-0">
              <Skeleton className="aspect-[16/10] w-full rounded-none" />
              <div className="space-y-2.5 p-4">
                <Skeleton className="h-5 w-2/3" />
                <Skeleton className="h-3.5 w-full" />
                <Skeleton className="h-3.5 w-4/5" />
              </div>
            </Card>
          ))}
        </div>
      ) : items.length === 0 ? (
        <EmptyState
          icon={Palette}
          title="No matching design styles"
          description="Try clearing your search query or switching theme/category filters."
          action={
            <Button variant="outline" size="sm" onClick={handleResetFilters}>
              Reset All Filters
            </Button>
          }
        />
      ) : (
        <div className="grid gap-5 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
          {items.map((style) => {
            const previewSrc =
              style.preview_url || `/api/styles/${encodeURIComponent(style.id)}/preview`
            return (
              <Card
                key={style.id}
                onClick={() => handleOpenStyle(style)}
                className="group cursor-pointer gap-0 overflow-hidden p-0 transition-all hover:-translate-y-0.5 hover:border-primary/50 hover:shadow-lg"
              >
                {/* Visual Preview Thumbnail */}
                <div className="relative aspect-[16/10] w-full overflow-hidden border-b border-border/60 bg-muted/30">
                  <img
                    src={previewSrc}
                    alt={style.title}
                    loading="lazy"
                    className="h-full w-full object-cover transition-transform duration-300 group-hover:scale-[1.03]"
                  />
                  <div className="absolute top-2.5 right-2.5 flex items-center gap-1.5">
                    <Badge
                      variant="outline"
                      className={cn(
                        "gap-1 border backdrop-blur-md text-[10px] font-semibold uppercase shadow-sm",
                        style.theme === "dark"
                          ? "border-violet-500/40 bg-zinc-950/80 text-violet-300"
                          : "border-amber-500/40 bg-zinc-950/75 text-amber-300"
                      )}
                    >
                      {style.theme === "dark" ? (
                        <Moon className="size-2.5" />
                      ) : (
                        <Sun className="size-2.5" />
                      )}
                      {style.theme}
                    </Badge>
                  </div>
                  {style.has_preview ? (
                    <span
                      className="absolute bottom-2 left-2 inline-flex items-center gap-1 rounded bg-background/80 px-1.5 py-0.5 text-[10px] font-medium text-foreground backdrop-blur-sm"
                      title="Real screenshot harvested"
                    >
                      <ImageIcon className="size-2.5 text-primary" />
                      Preview
                    </span>
                  ) : null}
                </div>

                {/* Card Body */}
                <div className="flex flex-1 flex-col justify-between gap-3 p-4">
                  <div className="space-y-1.5">
                    <div className="flex items-start justify-between gap-2">
                      <h3 className="line-clamp-1 font-semibold tracking-tight text-foreground group-hover:text-primary">
                        {style.title}
                      </h3>
                    </div>
                    <p className="line-clamp-2 text-xs leading-relaxed text-muted-foreground">
                      {style.tagline || "Distilled design system & UI specification"}
                    </p>
                    {style.snippet ? (
                      <p className="mt-1 line-clamp-2 rounded bg-muted/60 px-2 py-1 font-mono text-[11px] text-primary/90">
                        {style.snippet}
                      </p>
                    ) : null}
                  </div>

                  <div className="flex items-center justify-between border-t border-border/50 pt-2.5 text-[11px] text-muted-foreground">
                    <Badge variant="secondary" className="text-[10px] font-normal">
                      {style.category_label}
                    </Badge>

                    <div className="flex items-center gap-2">
                      {style.source_url ? (
                        <a
                          href={style.source_url}
                          target="_blank"
                          rel="noopener noreferrer"
                          onClick={(e) => e.stopPropagation()}
                          className="inline-flex items-center gap-1 text-muted-foreground hover:text-primary"
                          title={`Open ${style.source_url}`}
                        >
                          <ExternalLink className="size-3" />
                        </a>
                      ) : null}
                      <span className="font-medium text-primary group-hover:underline">
                        Inspect →
                      </span>
                    </div>
                  </div>
                </div>
              </Card>
            )
          })}
        </div>
      )}

      {/* Bottom Pagination */}
      {!debouncedQuery && totalPages > 1 ? (
        <div className="flex items-center justify-center gap-3 pt-4">
          <Button
            variant="outline"
            size="sm"
            disabled={page <= 0}
            onClick={() => {
              setPage((p) => Math.max(0, p - 1))
              window.scrollTo({ top: 0, behavior: "smooth" })
            }}
          >
            <ChevronLeft className="size-4" />
            Previous Page
          </Button>
          <span className="font-mono text-xs text-muted-foreground">
            Page {page + 1} of {totalPages}
          </span>
          <Button
            variant="outline"
            size="sm"
            disabled={page + 1 >= totalPages}
            onClick={() => {
              setPage((p) => p + 1)
              window.scrollTo({ top: 0, behavior: "smooth" })
            }}
          >
            Next Page
            <ChevronRight className="size-4" />
          </Button>
        </div>
      ) : null}

      {/* Detail Modal */}
      <StyleDetailDialog
        styleId={selectedStyleId}
        open={detailOpen}
        onOpenChange={setDetailOpen}
      />
    </main>
  )
}
