import { Header } from "./Header"
import { Footer } from "./Footer"

interface MainLayoutProps {
  children: React.ReactNode
}

export function MainLayout({ children }: MainLayoutProps) {
  return (
    <div className="flex min-h-screen flex-col">
      <Header />
      <main className="flex-1">
        <div className="container py-6">
          {children}
        </div>
      </main>
      <Footer />
    </div>
  )
}