import type { ReactNode } from 'react'
import type { LucideIcon } from 'lucide-react'

export function PageHeader({ title, description, actions, icon: Icon }: { title: string; description?: string; actions?: ReactNode; icon?: LucideIcon }) {
  return (
    <div className="flex flex-wrap items-center justify-between gap-2">
      <div className="min-w-0">
        <h1 className="flex items-center gap-2 truncate text-lg font-bold tracking-tight sm:text-2xl">
          {Icon && <Icon className="size-5 shrink-0 text-primary" />}
          {title}
        </h1>
        {description && <p className="mt-0.5 text-xs text-muted-foreground sm:text-sm">{description}</p>}
      </div>
      {actions && <div className="flex min-w-0 max-w-full flex-wrap items-center gap-1.5 sm:gap-2">{actions}</div>}
    </div>
  )
}
