export interface StyleShot {
  n: number
  url: string
  filename?: string
  ext?: string
  kind?: "image" | "video"
}

export interface StyleItem {
  id: string
  url: string
  title: string
  tagline: string
  theme: "light" | "dark" | string
  source_url: string
  category: string
  category_label: string
  has_design: boolean
  has_meta: boolean
  has_preview: boolean
  preview_url: string | null
  snippet?: string
  score?: number
}

export interface StyleDetail extends StyleItem {
  design_md: string
  meta: Record<string, unknown>
  shots?: StyleShot[]
}

export interface CategoryFacet {
  id: string
  label: string
  count: number
}

export interface StylesListResponse {
  total: number
  offset: number
  limit: number
  items: StyleItem[]
  facets: {
    themes: Record<string, number>
    categories: CategoryFacet[]
  }
}

export interface SearchResponse {
  query: string
  total_styles: number
  styles: StyleItem[]
  total_templates: number
  templates: TemplateItem[]
}

export interface CollectionItem {
  id: string
  name: string
  type: "category" | "theme" | "sitemap" | string
  description: string
  accent: string
  count: number
  sample_styles: StyleItem[]
}

export interface CollectionDetailResponse extends CollectionItem {
  offset: number
  limit: number
  styles: StyleItem[]
}

export interface TemplateItem {
  id: string
  num: number
  lang: "en" | "vi" | string
  title: string
  filename: string
  has_thumb: boolean
  has_prompt: boolean
  prompt_excerpt: string
  html_url: string
  thumb_url: string
}

export interface TemplateDetail extends TemplateItem {
  prompt: string
}

export interface TemplatesListResponse {
  total: number
  offset: number
  limit: number
  items: TemplateItem[]
  facets: {
    languages: Record<string, number>
  }
}

export interface ComponentItem {
  id: string
  name: string
  category: string
  kind: "html" | "tsx" | string
  description: string
  tags: string[]
  license: string
  source_url: string
  author: string
  files: string[]
  has_html: boolean
  html_url: string | null
}

export interface ComponentDetail extends ComponentItem {
  meta?: Record<string, unknown>
  sources?: Record<string, string>
}

export interface ComponentsListResponse {
  total: number
  offset: number
  limit: number
  items: ComponentItem[]
  facets: {
    categories: { id: string; count: number }[]
  }
}

export interface LicenseCorpusInfo {
  id: string
  name: string
  source: string
  license: string
  count: number
  with_design_md?: number
  languages?: Record<string, number>
  attribution_note: string
  terms_summary: string[]
}

export interface AttributedSource {
  id: string
  title: string
  source_url: string
  refero_url: string
  theme: string
  category: string
}

export interface LicensesResponse {
  service: string
  version: string
  corpora: LicenseCorpusInfo[]
  attributed_sources_total: number
  attributed_sources_sample: AttributedSource[]
}

export interface CorpusStatus {
  status: string
  service: string
  version: string
  indexed_at: number | null
  styles_count: number
  designs_count: number
  previews_count?: number
  collections_count: number
  templates_count: number
  components_count?: number
  refero_dir: string
  refero_dir_exists: boolean
  prompt_ui_dir: string
  prompt_ui_dir_exists: boolean
  themes: Record<string, number>
  categories: CategoryFacet[]
}

export interface AdminSettingsResponse {
  service_name: string
  service_version: string
  host: string
  port: number
  public_base_url: string
  data_root: string
  refero_dir: string
  refero_dir_exists: boolean
  prompt_ui_dir: string
  prompt_ui_dir_exists: boolean
  api_key_configured: boolean
  api_key_masked: string
  cors_origins: string
}
