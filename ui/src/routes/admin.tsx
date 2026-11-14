import { createFileRoute } from "@tanstack/react-router"
import { SettingsView } from "@/components/settings/SettingsView"

export const Route = createFileRoute("/admin")({
  component: SettingsView,
})
