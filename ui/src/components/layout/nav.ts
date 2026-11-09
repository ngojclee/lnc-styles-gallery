import {
  FileCode2,
  FolderKanban,
  Palette,
  Scale,
  Settings2,
} from "lucide-react"

export interface NavItem {
  title: string
  to: "/" | "/styles" | "/collections" | "/templates" | "/licenses" | "/admin"
  icon: typeof Palette
  description: string
  badgeKey?: "styles_count" | "collections_count" | "templates_count"
}

export interface NavGroup {
  label: string
  items: NavItem[]
}

export const NAV_GROUPS: NavGroup[] = [
  {
    label: "Style Library",
    items: [
      {
        title: "Design Styles",
        to: "/styles",
        icon: Palette,
        description: "Curated real-world design systems & DESIGN.md guides",
        badgeKey: "styles_count",
      },
      {
        title: "Collections",
        to: "/collections",
        icon: FolderKanban,
        description: "Grouped style sets by industry, mood, and color theme",
        badgeKey: "collections_count",
      },
      {
        title: "HTML Templates",
        to: "/templates",
        icon: FileCode2,
        description: "100 prompt-driven single-file HTML/CSS/JS templates",
        badgeKey: "templates_count",
      },
    ],
  },
  {
    label: "Governance & Ops",
    items: [
      {
        title: "Licenses & Sources",
        to: "/licenses",
        icon: Scale,
        description: "Source attributions & open-source corpus licenses",
      },
      {
        title: "Corpus Admin",
        to: "/admin",
        icon: Settings2,
        description: "Hot re-index, corpus diagnostics, and runtime settings",
      },
    ],
  },
]
