import { queryOptions } from "@tanstack/react-query"
import type {
  AdminSettingsResponse,
  AuthSessionResponse,
  CollectionDetailResponse,
  CollectionItem,
  ComponentDetail,
  ComponentsListResponse,
  CorpusStatus,
  LicensesResponse,
  PublicApiKeyItem,
  SearchResponse,
  StyleDetail,
  StyleShot,
  StylesListResponse,
  TemplateDetail,
  TemplatesListResponse,
} from "@/types/styles"

export class ApiHttpError extends Error {
  status: number

  constructor(status: number, message: string) {
    super(message)
    this.name = "ApiHttpError"
    this.status = status
  }
}

async function fetchJson<T>(url: string, init?: RequestInit): Promise<T> {
  const res = await fetch(url, {
    credentials: "same-origin",
    ...init,
    headers: {
      Accept: "application/json",
      ...(init?.headers ?? {}),
    },
  })
  if (!res.ok) {
    const text = await res.text().catch(() => "")
    let detail = text
    try {
      const parsed = JSON.parse(text) as { detail?: string; error?: string }
      detail = parsed.detail || parsed.error || text
    } catch {
      // keep raw text
    }
    throw new ApiHttpError(res.status, detail || `HTTP ${res.status}: ${res.statusText}`)
  }
  return res.json() as Promise<T>
}

export interface FetchStylesParams {
  q?: string
  theme?: string
  category?: string
  collection?: string
  has_design?: boolean
  limit?: number
  offset?: number
}

export async function fetchStyles(params: FetchStylesParams = {}): Promise<StylesListResponse> {
  const sp = new URLSearchParams()
  if (params.q) sp.set("q", params.q)
  if (params.theme) sp.set("theme", params.theme)
  if (params.category) sp.set("category", params.category)
  if (params.collection) sp.set("collection", params.collection)
  if (params.has_design !== undefined) sp.set("has_design", String(params.has_design))
  sp.set("limit", String(params.limit ?? 48))
  sp.set("offset", String(params.offset ?? 0))
  return fetchJson<StylesListResponse>(`/api/styles?${sp.toString()}`)
}

export async function fetchSearchStyles(params: {
  q: string
  theme?: string
  category?: string
  include_body?: boolean
  limit?: number
}): Promise<SearchResponse> {
  const sp = new URLSearchParams({
    q: params.q,
    include_body: String(params.include_body ?? true),
    limit: String(params.limit ?? 60),
  })
  if (params.theme) sp.set("theme", params.theme)
  if (params.category) sp.set("category", params.category)
  return fetchJson<SearchResponse>(`/api/search?${sp.toString()}`)
}

export async function fetchStyleDetail(styleId: string): Promise<StyleDetail> {
  return fetchJson<StyleDetail>(`/api/styles/${encodeURIComponent(styleId)}`)
}

export async function fetchStyleShots(styleId: string): Promise<StyleShot[]> {
  return fetchJson<StyleShot[]>(`/api/styles/${encodeURIComponent(styleId)}/shots`)
}

export async function fetchCollections(): Promise<{ total: number; items: CollectionItem[] }> {
  return fetchJson<{ total: number; items: CollectionItem[] }>("/api/collections")
}

export async function fetchCollectionDetail(
  collectionId: string,
  limit = 60,
  offset = 0
): Promise<CollectionDetailResponse> {
  const sp = new URLSearchParams({ limit: String(limit), offset: String(offset) })
  return fetchJson<CollectionDetailResponse>(
    `/api/collections/${encodeURIComponent(collectionId)}?${sp.toString()}`
  )
}

export async function fetchTemplates(params: {
  lang?: string
  q?: string
  limit?: number
  offset?: number
} = {}): Promise<TemplatesListResponse> {
  const sp = new URLSearchParams()
  if (params.lang) sp.set("lang", params.lang)
  if (params.q) sp.set("q", params.q)
  sp.set("limit", String(params.limit ?? 100))
  sp.set("offset", String(params.offset ?? 0))
  return fetchJson<TemplatesListResponse>(`/api/templates?${sp.toString()}`)
}

export async function fetchTemplateDetail(templateId: string): Promise<TemplateDetail> {
  return fetchJson<TemplateDetail>(`/api/templates/${encodeURIComponent(templateId)}`)
}

export async function fetchComponents(params: {
  category?: string
  kind?: string
  q?: string
  limit?: number
  offset?: number
} = {}): Promise<ComponentsListResponse> {
  const sp = new URLSearchParams()
  if (params.category) sp.set("category", params.category)
  if (params.kind) sp.set("kind", params.kind)
  if (params.q) sp.set("q", params.q)
  sp.set("limit", String(params.limit ?? 200))
  sp.set("offset", String(params.offset ?? 0))
  return fetchJson<ComponentsListResponse>(`/api/components?${sp.toString()}`)
}

export async function fetchComponentDetail(
  componentId: string,
  includeSource = false
): Promise<ComponentDetail> {
  const sp = new URLSearchParams({ include_source: String(includeSource) })
  return fetchJson<ComponentDetail>(`/api/components/${encodeURIComponent(componentId)}?${sp.toString()}`)
}

export async function fetchLicenses(): Promise<LicensesResponse> {
  return fetchJson<LicensesResponse>("/api/licenses")
}

export async function fetchCorpusStatus(): Promise<CorpusStatus> {
  return fetchJson<CorpusStatus>("/api/health")
}

export async function fetchAdminStatus(): Promise<CorpusStatus> {
  return fetchJson<CorpusStatus>("/api/admin/status")
}

export async function fetchAdminSettings(): Promise<AdminSettingsResponse> {
  return fetchJson<AdminSettingsResponse>("/api/admin/settings")
}

export async function triggerReindex(): Promise<{ status: string; summary: CorpusStatus }> {
  return fetchJson<{ status: string; summary: CorpusStatus }>("/api/admin/reindex", {
    method: "POST",
  })
}

export async function fetchAuthSession(): Promise<AuthSessionResponse> {
  return fetchJson<AuthSessionResponse>("/api/auth/session")
}

export async function loginWithManagementKey(key: string): Promise<{
  ok: boolean
  authenticated: boolean
  mode: string
  session_epoch: number
}> {
  return fetchJson("/api/auth/login", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ key }),
  })
}

export async function logoutSession(): Promise<{ ok: boolean; authenticated: boolean }> {
  return fetchJson("/api/auth/logout", {
    method: "POST",
  })
}

export async function revokeAllSessions(): Promise<{
  ok: boolean
  revoked: boolean
  session_epoch: number
}> {
  return fetchJson("/api/auth/revoke-all", {
    method: "POST",
  })
}

export async function fetchApiKeysList(): Promise<{ ok: boolean; keys: PublicApiKeyItem[] }> {
  return fetchJson<{ ok: boolean; keys: PublicApiKeyItem[] }>("/api/keys")
}

export async function createApiKey(payload: {
  label: string
  custom_secret?: string
}): Promise<{ ok: boolean; key: string; item: PublicApiKeyItem }> {
  return fetchJson("/api/keys", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(payload),
  })
}

export async function rotateApiKey(keyId: string): Promise<{
  ok: boolean
  key: string
  item: PublicApiKeyItem
}> {
  return fetchJson(`/api/keys/${encodeURIComponent(keyId)}/rotate`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({}),
  })
}

export async function setApiKeyRevoked(
  keyId: string,
  revoke: boolean
): Promise<{ ok: boolean; item: PublicApiKeyItem }> {
  return fetchJson(`/api/keys/${encodeURIComponent(keyId)}/revoke`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ revoke }),
  })
}

export async function deleteApiKey(keyId: string): Promise<{ ok: boolean; deleted: string }> {
  return fetchJson(`/api/keys/${encodeURIComponent(keyId)}`, {
    method: "DELETE",
  })
}

export const authSessionQuery = queryOptions({
  queryKey: ["auth-session"],
  queryFn: fetchAuthSession,
  staleTime: 15_000,
})

export const healthQuery = queryOptions({
  queryKey: ["health"],
  queryFn: fetchCorpusStatus,
  staleTime: 30_000,
})

export const collectionsQuery = queryOptions({
  queryKey: ["collections"],
  queryFn: fetchCollections,
  staleTime: 30_000,
})

export const licensesQuery = queryOptions({
  queryKey: ["licenses"],
  queryFn: fetchLicenses,
  staleTime: 60_000,
})
