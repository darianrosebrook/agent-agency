"use client";

/**
 * 404 Not Found Page
 * 
 * This page is displayed when a user navigates to a route that doesn't exist.
 */

import Link from "next/link";
import { Home, ArrowLeft } from "lucide-react";

export default function NotFound() {
  return (
    <div className="min-h-screen flex items-center justify-center bg-[#0d0d0d] p-8">
      <div className="text-center max-w-2xl">
        <div className="mb-8">
          <h1 className="text-9xl font-bold text-white mb-4">404</h1>
          <h2 className="text-3xl font-semibold text-white mb-4">Page Not Found</h2>
          <p className="text-gray-400 text-lg mb-8">
            The page you're looking for doesn't exist or has been moved.
          </p>
        </div>

        <div className="flex items-center justify-center gap-4">
          <Link
            href="/"
            className="flex items-center gap-2 px-6 py-3 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors font-medium"
          >
            <Home className="w-4 h-4" />
            Go to Dashboard
          </Link>
          <button
            onClick={() => window.history.back()}
            className="flex items-center gap-2 px-6 py-3 bg-[#1a1a1a] border border-gray-800 text-gray-300 rounded-lg hover:bg-gray-800 transition-colors"
          >
            <ArrowLeft className="w-4 h-4" />
            Go Back
          </button>
        </div>

        <div className="mt-12 text-left bg-[#1a1a1a] border border-gray-800 rounded-lg p-6">
          <h3 className="text-lg font-semibold text-white mb-4">Available Pages</h3>
          <ul className="space-y-2 text-gray-300">
            <li>
              <Link href="/" className="text-blue-500 hover:text-blue-400">
                Dashboard
              </Link>
            </li>
            <li>
              <Link href="/projects" className="text-blue-500 hover:text-blue-400">
                Projects
              </Link>
            </li>
            <li>
              <Link href="/chat" className="text-blue-500 hover:text-blue-400">
                Chat
              </Link>
            </li>
            <li>
              <Link href="/phase-planner" className="text-blue-500 hover:text-blue-400">
                Phase Planner
              </Link>
            </li>
            <li>
              <Link href="/agent-stats" className="text-blue-500 hover:text-blue-400">
                Agent Stats
              </Link>
            </li>
            <li>
              <Link href="/rules-governance" className="text-blue-500 hover:text-blue-400">
                Rules & Governance
              </Link>
            </li>
            <li>
              <Link href="/agent-health" className="text-blue-500 hover:text-blue-400">
                Agent Health
              </Link>
            </li>
            <li>
              <Link href="/settings" className="text-blue-500 hover:text-blue-400">
                Settings
              </Link>
            </li>
          </ul>
        </div>
      </div>
    </div>
  );
}

