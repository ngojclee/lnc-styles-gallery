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
      } else if (rawHash === "components") {
        navigate({ to: "/components" })
      } else if (rawHash === "platforms") {
        navigate({ to: "/platforms" })
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
          <main className="@container/main flex min-w-0 flex-1 flex-col px-4 pt-4 pb-10 sm:px-6 md:px-8">
            <div className="mx-auto flex min-w-0 w-full max-w-7xl flex-1 flex-col gap-4 md:gap-6">
              <Outlet />
            </div>
          </main>
        </SidebarInset>
      </SidebarProvider>
      <Toaster richColors closeButton position="bottom-right" />
    </TooltipProvider>
  )
}
