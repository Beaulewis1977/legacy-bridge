export function Footer() {
  return (
    <footer className="border-t">
      <div className="container flex flex-col gap-2 py-6 md:flex-row md:items-center md:justify-between">
        <div className="text-sm text-muted-foreground">
          © 2024 LegacyBridge. All rights reserved.
        </div>
        <nav className="flex gap-4 text-sm text-muted-foreground">
          <a href="/privacy" className="hover:underline">
            Privacy Policy
          </a>
          <a href="/terms" className="hover:underline">
            Terms of Service
          </a>
        </nav>
      </div>
    </footer>
  )
}