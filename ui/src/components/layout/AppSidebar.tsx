import { Link, useRouterState } from "@tanstack/react-router"
import { useQuery } from "@tanstack/react-query"
import { ExternalLink, Sparkles, Terminal } from "lucide-react"

import { healthQuery } from "@/api/styles"
import { NAV_GROUPS } from "@/components/layout/nav"
import { Badge } from "@/components/ui/badge"
import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarGroup,
  SidebarGroupContent,
  SidebarGroupLabel,
  SidebarHeader,
  SidebarMenu,
  SidebarMenuBadge,
  SidebarMenuButton,
  SidebarMenuItem,
  SidebarRail,
} from "@/components/ui/sidebar"

export function AppSidebar() {
  const pathname = useRouterState({ select: (s) => s.location.pathname })
  const { data: health } = useQuery(healthQuery)

  const isRouteActive = (to: string) => {
    if (to === "/styles") {
      return pathname === "/" || pathname.startsWith("/styles")
    }
    if (to === "/keys") {
      return (
        pathname === "/keys" ||
        pathname.startsWith("/keys/") ||
        pathname === "/api-keys" ||
        pathname.startsWith("/api-keys/")
      )
    }
    if (to === "/settings") {
      return (
        pathname === "/settings" ||
        pathname.startsWith("/settings/") ||
        pathname === "/admin" ||
        pathname.startsWith("/admin/")
      )
    }
    return pathname === to || pathname.startsWith(`${to}/`)
  }

  return (
    <Sidebar collapsible="icon" className="border-r border-sidebar-border/80">
      <SidebarHeader className="border-b border-sidebar-border/60 px-3 py-3">
        <SidebarMenu>
          <SidebarMenuItem>
            <SidebarMenuButton
              size="lg"
              asChild
              tooltip="LNC Style Library"
            >
              <Link to="/styles">
              <div className="flex aspect-square size-8 items-center justify-center overflow-hidden rounded-lg shadow-sm">
                <img src="/favicon.svg" alt="LNC Style Gallery" className="size-full" />
              </div>
              <div className="grid flex-1 text-left text-sm leading-tight">
                <span className="truncate font-bold tracking-tight">LNC STYLE GALLERY</span>
                <span className="truncate text-[11px] text-muted-foreground">
                  styles.lengoc.me Hub
                </span>
              </div>
              </Link>
            </SidebarMenuButton>
          </SidebarMenuItem>
        </SidebarMenu>
      </SidebarHeader>

      <SidebarContent>
        {NAV_GROUPS.map((group) => (
          <SidebarGroup key={group.label}>
            <SidebarGroupLabel className="text-[10px] font-semibold uppercase tracking-wider text-muted-foreground/80">
              {group.label}
            </SidebarGroupLabel>
            <SidebarGroupContent>
              <SidebarMenu>
                {group.items.map((item) => {
                  const Icon = item.icon
                  const active = isRouteActive(item.to)
                  const badgeCount = item.badgeKey && health ? health[item.badgeKey] : undefined
                  return (
                    <SidebarMenuItem key={item.to}>
                      <SidebarMenuButton
                        isActive={active}
                        tooltip={item.title}
                        asChild
                      >
                        <Link to={item.to}>
                          <Icon className="size-4" />
                          <span>{item.title}</span>
                        </Link>
                      </SidebarMenuButton>
                      {badgeCount !== undefined ? (
                        <SidebarMenuBadge className="font-mono text-[10px] text-muted-foreground">
                          {badgeCount}
                        </SidebarMenuBadge>
                      ) : null}
                    </SidebarMenuItem>
                  )
                })}
              </SidebarMenu>
            </SidebarGroupContent>
          </SidebarGroup>
        ))}
      </SidebarContent>

      <SidebarFooter className="border-t border-sidebar-border/60 p-3 group-data-[collapsible=icon]:hidden">
        <div className="rounded-xl border border-border/60 bg-muted/30 p-3 text-xs">
          <div className="mb-1.5 flex items-center justify-between">
            <span className="flex items-center gap-1.5 font-semibold text-foreground">
              <Terminal className="size-3.5 text-primary" />
              MCP Hub Ready
            </span>
            <Badge variant="outline" className="px-1.5 py-0 font-mono text-[10px]">
              v{health?.version ?? "0.2.0"}
            </Badge>
          </div>
          <p className="mb-2 text-[11px] leading-relaxed text-muted-foreground">
            Streamable HTTP endpoint <code className="text-foreground">/mcp</code> exposes{" "}
            <code className="text-primary">styles__*</code> tools.
          </p>
          <div className="flex items-center justify-between text-[11px]">
            <a
              href="/docs"
              target="_blank"
              rel="noopener noreferrer"
              className="inline-flex items-center gap-1 text-primary hover:underline"
            >
              OpenAPI Docs
              <ExternalLink className="size-3" />
            </a>
            <span className="inline-flex items-center gap-1 text-emerald-400">
              <Sparkles className="size-3" />
              Online
            </span>
          </div>
        </div>
      </SidebarFooter>

      <SidebarRail />
    </Sidebar>
  )
}
