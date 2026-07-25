import { Tabs, TabsList, TabsTrigger, TabsContent } from '@/components/ui/tabs'
import RulesPage from '@/pages/RulesPage'
import LogsPage from '@/pages/LogsPage'
import { Toaster } from 'sonner'
import { Network } from 'lucide-react'

export default function App() {
  return (
    <div className="min-h-screen bg-zinc-50">
      <Toaster richColors position="top-right" />
      <header className="border-b border-zinc-200 bg-white px-6 py-4">
        <div className="max-w-6xl mx-auto flex items-center gap-2">
          <Network className="h-5 w-5 text-zinc-700" />
          <h1 className="text-lg font-semibold text-zinc-900">Port Forward Manager</h1>
        </div>
      </header>
      <main className="max-w-6xl mx-auto px-6 py-6">
        <Tabs defaultValue="rules">
          <TabsList className="mb-4">
            <TabsTrigger value="rules">Rules</TabsTrigger>
            <TabsTrigger value="logs">Logs</TabsTrigger>
          </TabsList>
          <TabsContent value="rules"><RulesPage /></TabsContent>
          <TabsContent value="logs"><LogsPage /></TabsContent>
        </Tabs>
      </main>
    </div>
  )
}
