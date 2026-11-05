import { useState } from "react"

import { ConversionPage } from "@/pages/conversion"
import { ExtractMetaPage } from "@/pages/extract-meta"
import { SettingsPage } from "@/pages/settings"
import { HistoryPage } from "@/pages/history"
import { SidebarInset, SidebarProvider, SidebarTrigger } from "@/shared/ui/sidebar"
import { TooltipProvider } from "@/shared/ui/tooltip"
import { AppSidebar, type AppPage } from "@/widgets/app-sidebar"

import "./styles/index.css"

function App() {
  const [page, setPage] = useState<AppPage>("conversion")

  return (
    <TooltipProvider>
      <SidebarProvider className="h-svh min-h-0 overflow-hidden">
        <AppSidebar activePage={page} onPageChange={setPage} />
        <SidebarInset className="min-w-0 max-w-full overflow-hidden">
          <header className="flex h-14 shrink-0 items-center border-b px-4">
            <SidebarTrigger aria-label="Переключить боковую панель" />
          </header>
          {page === "conversion" && <ConversionPage />}
          {page === "extract-meta" && <ExtractMetaPage />}
          {page === "settings" && <SettingsPage />}
          {page === "history" && <HistoryPage />}
        </SidebarInset>
      </SidebarProvider>
    </TooltipProvider>
  )
}

export default App
