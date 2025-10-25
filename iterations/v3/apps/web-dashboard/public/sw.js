/**
 * Service Worker for Agent Agency Dashboard
 * Provides offline support and caching strategies
 */

const CACHE_NAME = 'agent-agency-dashboard-v1';
const STATIC_CACHE = 'static-v1';
const DYNAMIC_CACHE = 'dynamic-v1';
const API_CACHE = 'api-v1';

// Files to cache immediately
const STATIC_ASSETS = [
  '/',
  '/tasks',
  '/analytics',
  '/settings',
  '/manifest.json',
  '/favicon.ico',
  '/fonts/CreatoDisplay-Variable.woff2',
  '/fonts/DMMono-Regular.woff2',
];

// API endpoints to cache
const API_ENDPOINTS = [
  '/api/health',
  '/api/metrics',
  '/api/tasks',
  '/api/alerts',
  '/api/slos',
];

// Install event - cache static assets
self.addEventListener('install', (event) => {
  console.log('Service Worker: Installing...');
  
  event.waitUntil(
    caches.open(STATIC_CACHE)
      .then((cache) => {
        console.log('Service Worker: Caching static assets');
        return cache.addAll(STATIC_ASSETS);
      })
      .then(() => {
        console.log('Service Worker: Static assets cached');
        return self.skipWaiting();
      })
      .catch((error) => {
        console.error('Service Worker: Failed to cache static assets', error);
      })
  );
});

// Activate event - clean up old caches
self.addEventListener('activate', (event) => {
  console.log('Service Worker: Activating...');
  
  event.waitUntil(
    caches.keys()
      .then((cacheNames) => {
        return Promise.all(
          cacheNames.map((cacheName) => {
            if (cacheName !== STATIC_CACHE && 
                cacheName !== DYNAMIC_CACHE && 
                cacheName !== API_CACHE) {
              console.log('Service Worker: Deleting old cache', cacheName);
              return caches.delete(cacheName);
            }
          })
        );
      })
      .then(() => {
        console.log('Service Worker: Activated');
        return self.clients.claim();
      })
  );
});

// Fetch event - implement caching strategies
self.addEventListener('fetch', (event) => {
  const { request } = event;
  const url = new URL(request.url);

  // Skip non-GET requests
  if (request.method !== 'GET') {
    return;
  }

  // Handle different types of requests
  if (url.pathname.startsWith('/api/')) {
    // API requests - cache with network-first strategy
    event.respondWith(handleApiRequest(request));
  } else if (url.pathname.startsWith('/_next/static/')) {
    // Static assets - cache first strategy
    event.respondWith(handleStaticRequest(request));
  } else if (url.pathname === '/' || url.pathname.startsWith('/tasks') || 
             url.pathname.startsWith('/analytics') || url.pathname.startsWith('/settings')) {
    // Pages - network first with fallback
    event.respondWith(handlePageRequest(request));
  } else {
    // Other requests - network first
    event.respondWith(handleOtherRequest(request));
  }
});

// API request handler - network first with cache fallback
async function handleApiRequest(request) {
  const cache = await caches.open(API_CACHE);
  
  try {
    // Try network first
    const networkResponse = await fetch(request);
    
    if (networkResponse.ok) {
      // Cache successful responses
      const responseClone = networkResponse.clone();
      cache.put(request, responseClone);
      return networkResponse;
    }
    
    throw new Error('Network response not ok');
  } catch (error) {
    console.log('Service Worker: Network failed, trying cache for', request.url);
    
    // Fallback to cache
    const cachedResponse = await cache.match(request);
    if (cachedResponse) {
      return cachedResponse;
    }
    
    // Return offline page for API requests
    return new Response(
      JSON.stringify({ 
        error: 'Offline', 
        message: 'This data is not available offline' 
      }),
      { 
        status: 503, 
        headers: { 'Content-Type': 'application/json' } 
      }
    );
  }
}

// Static asset handler - cache first
async function handleStaticRequest(request) {
  const cache = await caches.open(STATIC_CACHE);
  
  // Try cache first
  const cachedResponse = await cache.match(request);
  if (cachedResponse) {
    return cachedResponse;
  }
  
  // Fallback to network
  try {
    const networkResponse = await fetch(request);
    if (networkResponse.ok) {
      cache.put(request, networkResponse.clone());
    }
    return networkResponse;
  } catch (error) {
    console.error('Service Worker: Failed to fetch static asset', request.url);
    throw error;
  }
}

// Page request handler - network first with cache fallback
async function handlePageRequest(request) {
  const cache = await caches.open(DYNAMIC_CACHE);
  
  try {
    // Try network first
    const networkResponse = await fetch(request);
    
    if (networkResponse.ok) {
      // Cache successful responses
      const responseClone = networkResponse.clone();
      cache.put(request, responseClone);
      return networkResponse;
    }
    
    throw new Error('Network response not ok');
  } catch (error) {
    console.log('Service Worker: Network failed, trying cache for', request.url);
    
    // Fallback to cache
    const cachedResponse = await cache.match(request);
    if (cachedResponse) {
      return cachedResponse;
    }
    
    // Fallback to offline page
    return cache.match('/offline.html') || new Response('Offline', { status: 503 });
  }
}

// Other request handler - network first
async function handleOtherRequest(request) {
  try {
    return await fetch(request);
  } catch (error) {
    console.error('Service Worker: Failed to fetch', request.url);
    throw error;
  }
}

// Background sync for offline actions
self.addEventListener('sync', (event) => {
  if (event.tag === 'background-sync') {
    console.log('Service Worker: Background sync triggered');
    event.waitUntil(doBackgroundSync());
  }
});

async function doBackgroundSync() {
  // Implement background sync logic here
  console.log('Service Worker: Performing background sync');
}

// Push notifications
self.addEventListener('push', (event) => {
  console.log('Service Worker: Push notification received');
  
  const options = {
    body: event.data ? event.data.text() : 'New notification',
    icon: '/favicon.ico',
    badge: '/favicon.ico',
    vibrate: [100, 50, 100],
    data: {
      dateOfArrival: Date.now(),
      primaryKey: 1
    },
    actions: [
      {
        action: 'explore',
        title: 'View Dashboard',
        icon: '/favicon.ico'
      },
      {
        action: 'close',
        title: 'Close',
        icon: '/favicon.ico'
      }
    ]
  };
  
  event.waitUntil(
    self.registration.showNotification('Agent Agency Dashboard', options)
  );
});

// Notification click handler
self.addEventListener('notificationclick', (event) => {
  console.log('Service Worker: Notification clicked');
  
  event.notification.close();
  
  if (event.action === 'explore') {
    event.waitUntil(
      clients.openWindow('/')
    );
  }
});

