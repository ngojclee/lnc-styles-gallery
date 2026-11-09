import { isValidElement, type ReactNode } from 'react'
import type { LucideIcon } from 'lucide-react'

export function StatCard({ title, label, value, icon, desc, hint }: { title?: string; label?: string; value: ReactNode; icon?: ReactNode | LucideIcon; desc?: string; hint?: string }) {
  const iconNode = isValidElement(icon) ? icon : icon ? (() => { const C = icon as LucideIcon; return <C /> })() : null
  return (
    <div className="min-h-[96px] rounded-[20px] bg-muted px-4 py-3">
      <div className="flex items-start justify-between gap-3">
        <div className="truncate text-xs font-medium text-muted-foreground">{title ?? label}</div>
        <div className="flex size-6 items-center justify-center rounded-full bg-background/70 text-muted-foreground [&_svg]:size-3.5">
          {iconNode}
        </div>
      </div>
      <div className="mt-3 truncate text-2xl leading-none font-semibold tracking-[-0.03em]">{value}</div>
      <div className="mt-2 truncate text-xs text-muted-foreground">{desc ?? hint ?? ' '}</div>
    </div>
  )
}
