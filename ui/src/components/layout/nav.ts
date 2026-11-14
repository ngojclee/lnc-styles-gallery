import {
  Blocks,
  FileCode2,
  FolderKanban,
  KeyRound,
  MonitorSmartphone,
  Palette,
  Plug,
  Scale,
  Settings2,
} from "lucide-react"

export interface NavItem {
  title: string
  to:
    | "/"
    | "/styles"
    | "/collections"
    | "/templates"
    | "/components"
    | "/platforms"
    | "/licenses"
    | "/keys"
    | "/mcp-connectors"
    | "/settings"
  icon: typeof Palette
  description: string
  badgeKey?: "styles_count" | "collections_count" | "templates_count" | "components_count"
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
      {
        title: "Components & Effects",
        to: "/components",
        icon: Blocks,
        description: "UI effect recipes, LNC demos & Knowns UI sources",
        badgeKey: "components_count",
      },
      {
        title: "Platforms",
        to: "/platforms",
        icon: MonitorSmartphone,
        description: "iOS / Android / vendor official design systems",
      },
      {
        title: "Licenses & Sources",
        to: "/licenses",
        icon: Scale,
        description: "Source attributions & open-source corpus licenses",
      },
    ],
  },
  {
    label: "Governance & Ops",
    items: [
      {
        title: "API Keys",
        to: "/keys",
        icon: KeyRound,
        description: "Inbound API keys (Copy, Rotate, Revoke, Remove)",
      },
      {
        title: "MCP Connectors",
        to: "/mcp-connectors",
        icon: Plug,
        description: "Streamable HTTP /mcp endpoint, 8 tools & connector tokens",
      },
      {
        title: "Settings",
        to: "/settings",
        icon: Settings2,
        description: "Authentication posture, corpus diagnostics & runtime URL",
      },
    ],
  },
]
