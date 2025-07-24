import { NextResponse } from 'next/server';
import type { NextRequest } from 'next/server';

export function middleware(request: NextRequest) {
  // Clone the request headers and set a new header
  const requestHeaders = new Headers(request.headers);
  
  // Create response
  const response = NextResponse.next({
    request: {
      headers: requestHeaders,
    },
  });
  
  // Content Security Policy
  const cspHeader = `
    default-src 'self';
    script-src 'self' 'unsafe-inline' 'unsafe-eval' https://cdn.jsdelivr.net;
    style-src 'self' 'unsafe-inline' https://fonts.googleapis.com;
    font-src 'self' https://fonts.gstatic.com;
    img-src 'self' data: https: blob:;
    connect-src 'self' https://api.github.com https://raw.githubusercontent.com;
    frame-src 'none';
    object-src 'none';
    base-uri 'self';
    form-action 'self';
    frame-ancestors 'none';
    upgrade-insecure-requests;
  `.replace(/\s{2,}/g, ' ').trim();
  
  // Security headers
  response.headers.set('Content-Security-Policy', cspHeader);
  response.headers.set('X-Content-Type-Options', 'nosniff');
  response.headers.set('X-Frame-Options', 'DENY');
  response.headers.set('X-XSS-Protection', '1; mode=block');
  response.headers.set('Referrer-Policy', 'strict-origin-when-cross-origin');
  response.headers.set('Permissions-Policy', 'camera=(), microphone=(), geolocation=()');
  
  // For Tauri app compatibility
  if (request.headers.get('user-agent')?.includes('Tauri')) {
    // Relax CSP for Tauri environment
    const tauriCsp = `
      default-src 'self' tauri: https://tauri.localhost;
      script-src 'self' 'unsafe-inline' 'unsafe-eval';
      style-src 'self' 'unsafe-inline';
      font-src 'self' data:;
      img-src 'self' data: https: blob: tauri: https://tauri.localhost;
      connect-src 'self' ipc: https://ipc.localhost tauri: https://tauri.localhost;
    `.replace(/\s{2,}/g, ' ').trim();
    
    response.headers.set('Content-Security-Policy', tauriCsp);
  }
  
  return response;
}

// Apply middleware to all routes except static files and API routes
export const config = {
  matcher: [
    /*
     * Match all request paths except for the ones starting with:
     * - api (API routes)
     * - _next/static (static files)
     * - _next/image (image optimization files)
     * - favicon.ico (favicon file)
     * - public folder
     */
    '/((?!api|_next/static|_next/image|favicon.ico|.*\\..*).*)' 
  ],
};