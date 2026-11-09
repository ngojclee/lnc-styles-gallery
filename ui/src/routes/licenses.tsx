import { useMemo, useState } from "react"
import { createFileRoute } from "@tanstack/react-router"
import { useQuery } from "@tanstack/react-query"
import { CheckCircle2, ExternalLink, Scale, Search, ShieldCheck } from "lucide-react"

import { licensesQuery } from "@/api/styles"
import { PageHeader } from "@/components/common/PageHeader"
import { Badge } from "@/components/ui/badge"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Input } from "@/components/ui/input"
import { Skeleton } from "@/components/ui/skeleton"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table"

export const Route = createFileRoute("/licenses")({
  component: LicensesPage,
})

function LicensesPage() {
  const { data, isLoading } = useQuery(licensesQuery)
  const [filterText, setFilterText] = useState("")

  const filteredSources = useMemo(() => {
    const list = data?.attributed_sources_sample ?? []
    const q = filterText.trim().toLowerCase()
    if (!q) return list
    return list.filter(
      (item) =>
        item.title.toLowerCase().includes(q) ||
        item.source_url.toLowerCase().includes(q) ||
        item.category.toLowerCase().includes(q)
    )
  }, [data?.attributed_sources_sample, filterText])

  return (
    <main className="flex-1 space-y-6 p-6">
      <PageHeader
        icon={Scale}
        title="Licenses & Source Attributions"
        description="Structured license manifest and product website attributions for all design corpora indexed by lnc-styles-gallery."
      />

      {isLoading ? (
        <div className="grid gap-5 lg:grid-cols-2">
          <Skeleton className="h-64 rounded-xl" />
          <Skeleton className="h-64 rounded-xl" />
        </div>
      ) : (
        <div className="grid gap-5 lg:grid-cols-2">
          {(data?.corpora ?? []).map((corpus) => (
            <Card key={corpus.id}>
              <CardHeader>
                <div className="flex flex-wrap items-start justify-between gap-2">
                  <div>
                    <CardTitle className="flex items-center gap-2 text-base">
                      <ShieldCheck className="size-4 text-primary" />
                      {corpus.name}
                    </CardTitle>
                    <CardDescription className="mt-1 text-xs">
                      {corpus.attribution_note}
                    </CardDescription>
                  </div>
                  <Badge variant="outline" className="font-mono text-xs">
                    {corpus.license}
                  </Badge>
                </div>
              </CardHeader>
              <CardContent className="space-y-4 text-xs">
                <div className="flex flex-wrap items-center gap-4 rounded-lg border border-border/60 bg-muted/30 px-3.5 py-2.5">
                  <div>
                    <span className="text-muted-foreground">Records Indexed: </span>
                    <strong className="font-mono text-foreground">{corpus.count}</strong>
                  </div>
                  {corpus.with_design_md !== undefined ? (
                    <div>
                      <span className="text-muted-foreground">With DESIGN.md: </span>
                      <strong className="font-mono text-foreground">
                        {corpus.with_design_md}
                      </strong>
                    </div>
                  ) : null}
                  <a
                    href={corpus.source}
                    target="_blank"
                    rel="noopener noreferrer"
                    className="ml-auto inline-flex items-center gap-1 font-medium text-primary hover:underline"
                  >
                    Upstream Source
                    <ExternalLink className="size-3" />
                  </a>
                </div>

                <ul className="space-y-2">
                  {corpus.terms_summary.map((term, i) => (
                    <li key={i} className="flex items-start gap-2 text-muted-foreground">
                      <CheckCircle2 className="mt-0.5 size-3.5 shrink-0 text-emerald-400" />
                      <span>{term}</span>
                    </li>
                  ))}
                </ul>
              </CardContent>
            </Card>
          ))}
        </div>
      )}

      {/* Attributed Product Websites Table */}
      <Card>
        <CardHeader>
          <div className="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
            <div>
              <CardTitle className="text-base">
                Attributed Product Websites ({data?.attributed_sources_total ?? 0} total)
              </CardTitle>
              <CardDescription className="text-xs">
                Each style record preserves its original product URL and Refero reference link.
              </CardDescription>
            </div>
            <div className="relative w-full sm:w-72">
              <Search className="pointer-events-none absolute top-1/2 left-3 size-3.5 -translate-y-1/2 text-muted-foreground" />
              <Input
                value={filterText}
                onChange={(e) => setFilterText(e.target.value)}
                placeholder="Filter attributed sources..."
                className="h-8 pl-8 text-xs"
              />
            </div>
          </div>
        </CardHeader>
        <CardContent>
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Product / Brand</TableHead>
                <TableHead>Category</TableHead>
                <TableHead>Theme</TableHead>
                <TableHead>Original Website</TableHead>
                <TableHead className="text-right">Refero Reference</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {filteredSources.slice(0, 100).map((src) => (
                <TableRow key={src.id}>
                  <TableCell className="font-medium text-foreground">{src.title}</TableCell>
                  <TableCell>
                    <Badge variant="secondary" className="text-[10px]">
                      {src.category}
                    </Badge>
                  </TableCell>
                  <TableCell className="capitalize text-muted-foreground">
                    {src.theme}
                  </TableCell>
                  <TableCell>
                    {src.source_url ? (
                      <a
                        href={src.source_url}
                        target="_blank"
                        rel="noopener noreferrer"
                        className="inline-flex items-center gap-1 text-xs text-primary hover:underline"
                      >
                        {src.source_url}
                        <ExternalLink className="size-3" />
                      </a>
                    ) : (
                      <span className="text-muted-foreground">—</span>
                    )}
                  </TableCell>
                  <TableCell className="text-right">
                    {src.refero_url ? (
                      <a
                        href={src.refero_url}
                        target="_blank"
                        rel="noopener noreferrer"
                        className="inline-flex items-center gap-1 text-xs text-muted-foreground hover:text-foreground"
                      >
                        Refero
                        <ExternalLink className="size-3" />
                      </a>
                    ) : null}
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </CardContent>
      </Card>
    </main>
  )
}
