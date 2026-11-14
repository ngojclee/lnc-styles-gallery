import { createFileRoute } from "@tanstack/react-router"

import { KeysPage } from "@/routes/keys"

export const Route = createFileRoute("/api-keys")({
  component: KeysPage,
})
