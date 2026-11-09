import { useEffect } from "react"
import { Outlet, createRootRouteWithContext, useNavigate } from "@tanstack/react-router"
import type { QueryClient } from "@tanstack/react-query"

import { AppHeader } from "@/components/layout/AppHeader"
import { AppSidebar } from "@/components/layout/AppSidebar"
import { SidebarInset, SidebarProvider } from "@/components/ui/sidebar"
import { Toaster } from "@/components/ui/sonner"
import { TooltipProvider } from "@/components/ui/tooltip"

interface RouterContext {
  queryClient: QueryClient
}

export const Route = createRootRouteWithContext<RouterContext>()({
  component: RootLayout,
})

function RootLayout() {
  const navigate = useNavigate()

  useEffect(() => {
    const syncHashRoute = () => {
      const rawHash = window.location.hash.replace(/^#\/?/, "").trim().toLowerCase()
      if (!rawHash) return
      if (rawHash === "styles" || rawHash.startsWith("styles?")) {
        navigate({ to: "/styles" })
      } else if (rawHash === "collections") {
        navigate({ to: "/collections" })
      } else if (rawHash === "templates") {
        navigate({ to: "/templates" })
      } else if (rawHash === "licenses") {
        navigate({ to: "/licenses" })
      } else if (rawHash === "admin") {
        navigate({ to: "/admin" })
      }
    }
    syncHashRoute()
    window.addEventListener("hashchange", syncHashRoute)
    return () => window.removeEventListener("hashchange", syncHashRoute)
  }, [navigate])

  return (
    <TooltipProvider delayDuration={200}>
      <SidebarProvider>
        <AppSidebar />
        <SidebarInset>
          <AppHeader />
          <div className="flex flex-1 flex-col">
            <Outlet />
          </div>
        </SidebarInset>
      </SidebarProvider>
      <Toaster richColors closeButton position="bottom-right" />
    </TooltipProvider>
  )
}
