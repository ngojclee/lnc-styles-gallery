import { createFileRoute } from "@tanstack/react-router"
import { StylesGalleryPage } from "@/routes/styles"

export const Route = createFileRoute("/")({
  component: StylesGalleryPage,
})
