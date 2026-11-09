import { useState } from "react"
import { createFileRoute } from "@tanstack/react-router"
import { useQuery } from "@tanstack/react-query"
import { MonitorSmartphone } from "lucide-react"

import { fetchStyles } from "@/api/styles"
import { EmptyState } from "@/components/common/EmptyState"
import { PageHeader } from "@/components/common/PageHeader"
import { StyleDetailDialog } from "@/components/common/StyleDetailDialog"
import { Badge } from "@/components/ui/badge"
import { Card } from "@/components/ui/card"
import { Skeleton } from "@/components/ui/skeleton"
import { cn } from "@/lib/utils"

export const Route = createFileRoute("/platforms")({
  component: PlatformsPage,
})

function PlatformsPage() {
  const [selectedStyleId, setSelectedStyleId] = useState<string | null>(null)
  const [detailOpen, setDetailOpen] = useState(false)

  const stylesQuery = useQuery({
    queryKey: ["platform-styles"],
    queryFn: () => fetchStyles({ category: "platform-system", limit: 100 }),
  })

  const items = stylesQuery.data?.items ?? []

  return (
    <main className="flex-1 space-y-6">
      <PageHeader
        icon={MonitorSmartphone}
        title="Platform Design Systems"
        description="Official vendor design systems — Apple HIG (iOS), Material Design 3 (Android), Fluent 2, Carbon, Polaris, Primer, Ant Design, shadcn/ui and more. Canonical tokens for building platform-native UIs."
      />

      {stylesQuery.isLoading ? (
        <div className="grid gap-5 sm:grid-cols-2 lg:grid-cols-3">
          {Array.from({ length: 6 }).map((_, idx) => (
            <Card key={idx} className="overflow-hidden p-0">
              <Skeleton className="aspect-[16/10] w-full rounded-none" />
              <div className="space-y-2 p-4">
                <Skeleton className="h-5 w-2/3" />
                <Skeleton className="h-3.5 w-full" />
              </div>
            </Card>
          ))}
        </div>
      ) : items.length === 0 ? (
        <EmptyState
          icon={MonitorSmartphone}
          title="No platform systems found"
          description="The platform-system collection is empty — run a re-index."
        />
      ) : (
        <div className="grid gap-5 sm:grid-cols-2 lg:grid-cols-3">
          {items.map((s) => (
            <Card
              key={s.id}
              onClick={() => {
                setSelectedStyleId(s.id)
                setDetailOpen(true)
              }}
              className="group cursor-pointer gap-0 overflow-hidden p-0 transition-all hover:-translate-y-0.5 hover:border-primary/50 hover:shadow-lg"
            >
              <div className="relative aspect-[16/10] w-full overflow-hidden border-b border-border/60 bg-muted/30">
                {s.preview_url ? (
                  <img
                    src={s.preview_url}
                    alt={s.title}
                    loading="lazy"
                    className="h-full w-full object-cover transition-transform duration-300 group-hover:scale-[1.03]"
                  />
                ) : (
                  <div className="flex h-full items-center justify-center text-muted-foreground">
                    <MonitorSmartphone className="size-8 opacity-40" />
                  </div>
                )}
                <Badge
                  variant="outline"
                  className={cn(
                    "absolute top-2.5 right-2.5 border backdrop-blur-md text-[10px] font-semibold uppercase",
                    s.theme === "dark"
                      ? "border-indigo-500/40 bg-zinc-950/80 text-indigo-300"
                      : "border-amber-500/40 bg-zinc-950/80 text-amber-300"
                  )}
                >
                  {s.theme}
                </Badge>
              </div>
              <div className="space-y-1.5 p-4">
                <h3 className="line-clamp-1 font-semibold tracking-tight text-foreground group-hover:text-primary">
                  {s.title}
                </h3>
                <p className="line-clamp-2 text-xs leading-relaxed text-muted-foreground">
                  {s.tagline || "Official platform design system"}
                </p>
              </div>
            </Card>
          ))}
        </div>
      )}

      <StyleDetailDialog
        styleId={selectedStyleId}
        open={detailOpen}
        onOpenChange={setDetailOpen}
      />
    </main>
  )
}
