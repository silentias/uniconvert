import { FileSearchIcon, HistoryIcon, RefreshCwIcon, SettingsIcon } from "lucide-react"

import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarGroup,
  SidebarGroupContent,
  SidebarGroupLabel,
  SidebarHeader,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
  SidebarRail,
} from "@/shared/ui/sidebar"

export type AppPage = "conversion" | "extract-meta" | "history" | "settings"

type AppSidebarProps = {
  activePage: AppPage
  onPageChange: (page: AppPage) => void
}

const navigation = [
  { id: "conversion", title: "Конвертация", icon: RefreshCwIcon },
  { id: "extract-meta", title: "Извлечение Meta", icon: FileSearchIcon },
  { id: "history", title: "История", icon: HistoryIcon },
  { id: "settings", title: "Настройки", icon: SettingsIcon },
] as const

export function AppSidebar({ activePage, onPageChange }: AppSidebarProps) {
  return (
    <Sidebar collapsible="icon">
      <SidebarHeader>
        <div className="flex h-8 items-center gap-2 px-2 font-semibold group-data-[collapsible=icon]:justify-center group-data-[collapsible=icon]:px-0">
          <img src="/uniconvert_icons/uniconvert_64x64.png" alt="" className="size-7 shrink-0 object-contain" />
          <span className="group-data-[collapsible=icon]:hidden">UniConvert</span>
        </div>
      </SidebarHeader>

      <SidebarContent>
        <SidebarGroup>
          <SidebarGroupLabel>Навигация</SidebarGroupLabel>
          <SidebarGroupContent>
            <SidebarMenu>
              {navigation.map((item) => (
                <SidebarMenuItem key={item.id}>
                  <SidebarMenuButton
                    isActive={activePage === item.id}
                    tooltip={item.title}
                    onClick={() => onPageChange(item.id)}
                  >
                    <item.icon />
                    <span>{item.title}</span>
                  </SidebarMenuButton>
                </SidebarMenuItem>
              ))}
            </SidebarMenu>
          </SidebarGroupContent>
        </SidebarGroup>
      </SidebarContent>

      <SidebarFooter />
      <SidebarRail />
    </Sidebar>
  )
}
