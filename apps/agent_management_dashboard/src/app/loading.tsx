/**
 * Global Loading Page
 * 
 * This page is displayed while the application is loading.
 */

export default function Loading() {
  return (
    <div className="min-h-screen flex items-center justify-center bg-[#0d0d0d]">
      <div className="text-center">
        <div className="inline-block w-12 h-12 border-4 border-gray-800 border-t-blue-600 rounded-full animate-spin mb-4"></div>
        <p className="text-gray-400">Loading...</p>
      </div>
    </div>
  );
}

