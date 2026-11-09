import { createFileRoute, useNavigate } from "@tanstack/react-router"
import { useQuery } from "@tanstack/react-query"
import { ArrowRight, FolderKanban, Layers, Sparkles } from "lucide-react"

import { collectionsQuery } from "@/api/styles"
import { EmptyState } from "@/components/common/EmptyState"
import { PageHeader } from "@/components/common/PageHeader"
import { StatCard } from "@/components/common/StatCard"
import { Badge } from "@/components/ui/badge"
import { Card } from "@/components/ui/card"
import { Skeleton } from "@/components/ui/skeleton"
import { setPendingStyleFilter } from "@/routes/styles"
import type { CollectionItem } from "@/types/styles"

export const Route = createFileRoute("/collections")({
  component: CollectionsPage,
})

function CollectionsPage() {
  const navigate = useNavigate()
  const { data, isLoading } = useQuery(collectionsQuery)
  const collections = data?.items ?? []

  const categoryCollections = collections.filter(
    (c) => c.type === "category" || c.type === "mood"
  )
  const themeCollections = collections.filter((c) => c.type === "theme")
  const sitemapCollections = collections.filter(
    (c) => c.type !== "category" && c.type !== "mood" && c.type !== "theme"
  )

  const handleSelectCollection = (col: CollectionItem) => {
    if (col.type === "category" || col.type === "mood") {
      setPendingStyleFilter({
        category: col.id.startsWith("cat-") ? col.id.slice(4) : col.id,
      })
    } else if (col.type === "theme" && col.id.startsWith("theme-")) {
      setPendingStyleFilter({ theme: col.id.slice(6) })
    } else {
      setPendingStyleFilter({ collectionId: col.id, collectionName: col.name })
    }
    navigate({ to: "/styles" })
  }

  return (
    <main className="flex-1 space-y-6 p-6">
      <PageHeader
        icon={FolderKanban}
        title="Style Collections"
        description="Curated groupings of design systems organized by product industry, aesthetic mood, and color theme."
      />

      <div className="grid gap-4 sm:grid-cols-3">
        <StatCard
          label="Total Collections"
          value={collections.length}
          hint="Derived taxonomy & sitemap groups"
          icon={FolderKanban}
        />
        <StatCard
          label="Industry & Mood Sets"
          value={categoryCollections.length}
          hint="AI, SaaS, Fintech, Editorial, Brutalist..."
          icon={Sparkles}
        />
        <StatCard
          label="Theme Collections"
          value={themeCollections.length}
          hint="Light & Dark mode design systems"
          icon={Layers}
        />
      </div>

      {isLoading ? (
        <div className="grid gap-5 sm:grid-cols-2 lg:grid-cols-3">
          {Array.from({ length: 9 }).map((_, idx) => (
            <Card key={idx} className="p-5">
              <Skeleton className="h-6 w-1/2" />
              <Skeleton className="h-4 w-full" />
              <div className="grid grid-cols-4 gap-2 pt-2">
                <Skeleton className="aspect-[16/10] rounded-md" />
                <Skeleton className="aspect-[16/10] rounded-md" />
                <Skeleton className="aspect-[16/10] rounded-md" />
                <Skeleton className="aspect-[16/10] rounded-md" />
              </div>
            </Card>
          ))}
        </div>
      ) : collections.length === 0 ? (
        <EmptyState
          icon={FolderKanban}
          title="No collections available"
          description="Re-index the Refero styles corpus to populate style collections."
        />
      ) : (
        <div className="space-y-8">
          <CollectionSection
            title="Industry & Aesthetic Mood Collections"
            items={categoryCollections}
            onSelect={handleSelectCollection}
          />

          {themeCollections.length > 0 ? (
            <CollectionSection
              title="Color Theme Collections"
              items={themeCollections}
              onSelect={handleSelectCollection}
            />
          ) : null}

          {sitemapCollections.length > 0 ? (
            <CollectionSection
              title="Curated Sitemap Collections"
              items={sitemapCollections}
              onSelect={handleSelectCollection}
            />
          ) : null}
        </div>
      )}
    </main>
  )
}

function CollectionSection({
  title,
  items,
  onSelect,
}: {
  title: string
  items: CollectionItem[]
  onSelect: (col: CollectionItem) => void
}) {
  return (
    <section className="space-y-3">
      <h2 className="text-sm font-semibold uppercase tracking-wider text-muted-foreground">
        {title}
      </h2>
      <div className="grid gap-5 sm:grid-cols-2 lg:grid-cols-3">
        {items.map((col) => (
          <Card
            key={col.id}
            onClick={() => onSelect(col)}
            className="group relative cursor-pointer gap-3 overflow-hidden p-5 transition-all hover:-translate-y-0.5 hover:border-primary/50 hover:shadow-lg"
          >
            <div
              className="absolute inset-x-0 top-0 h-1"
              style={{ backgroundColor: col.accent || "#7c5cff" }}
            />
            <div className="flex items-start justify-between gap-2 pt-1">
              <div className="space-y-1">
                <h3 className="font-semibold tracking-tight text-foreground group-hover:text-primary">
                  {col.name}
                </h3>
                <p className="line-clamp-2 text-xs text-muted-foreground">
                  {col.description}
                </p>
              </div>
              <Badge variant="secondary" className="shrink-0 font-mono text-xs">
                {col.count} styles
              </Badge>
            </div>

            {/* 4-Thumbnail Sample Preview Strip */}
            {col.sample_styles && col.sample_styles.length > 0 ? (
              <div className="grid grid-cols-4 gap-2 pt-2">
                {col.sample_styles.slice(0, 4).map((sample) => (
                  <div
                    key={sample.id}
                    className="aspect-[16/10] overflow-hidden rounded-md border border-border/60 bg-muted/40"
                  >
                    <img
                      src={
                        sample.preview_url ||
                        `/api/styles/${encodeURIComponent(sample.id)}/preview`
                      }
                      alt={sample.title}
                      loading="lazy"
                      className="h-full w-full object-cover"
                    />
                  </div>
                ))}
              </div>
            ) : null}

            <div className="flex items-center justify-between border-t border-border/50 pt-3 text-xs">
              <span className="capitalize text-muted-foreground">{col.type} set</span>
              <span className="inline-flex items-center gap-1 font-medium text-primary group-hover:underline">
                Explore Styles
                <ArrowRight className="size-3.5" />
              </span>
            </div>
          </Card>
        ))}
      </div>
    </section>
  )
}
