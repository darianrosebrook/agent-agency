#!/usr/bin/env python3
"""
Simple HTTP server for Agent Agency V3 Web Interface
Serves static files with proper CORS headers for API communication
"""

import http.server
import socketserver
import os
import sys
from urllib.parse import unquote
import mimetypes

class CORSHTTPRequestHandler(http.server.SimpleHTTPRequestHandler):
    """HTTP request handler with CORS support"""

    def end_headers(self):
        # Add CORS headers for API communication
        self.send_header('Access-Control-Allow-Origin', '*')
        self.send_header('Access-Control-Allow-Methods', 'GET, POST, PUT, DELETE, OPTIONS')
        self.send_header('Access-Control-Allow-Headers', 'Content-Type, Authorization')
        super().end_headers()

    def do_OPTIONS(self):
        """Handle preflight CORS requests"""
        self.send_response(200)
        self.end_headers()

    def log_message(self, format, *args):
        """Override logging to be less verbose"""
        if "GET /api/" in format or "POST /api/" in format:
            # Log API calls
            super().log_message(format, *args)
        elif "GET /" in format and not any(ext in format for ext in ['.ico', '.png', '.jpg', '.css']):
            # Log main page loads but not assets
            super().log_message("Web Interface: %s", format % args)
        # Suppress other asset requests

def main():
    """Main server function"""
    port = 3000

    # Check if port is provided as argument
    if len(sys.argv) > 1:
        try:
            port = int(sys.argv[1])
        except ValueError:
            print(f"Invalid port number: {sys.argv[1]}")
            sys.exit(1)

    # Change to the web-app directory
    web_dir = os.path.dirname(os.path.abspath(__file__))
    os.chdir(web_dir)

    # Configure the server
    with socketserver.TCPServer(("", port), CORSHTTPRequestHandler) as httpd:
        print(f"🚀 Agent Agency V3 Web Interface")
        print(f"📍 Server running at: http://localhost:{port}")
        print(f"🔗 API Server expected at: http://localhost:8080")
        print(f"📁 Serving files from: {web_dir}")
        print(f"⏹️  Press Ctrl+C to stop")
        print()

        try:
            httpd.serve_forever()
        except KeyboardInterrupt:
            print("\n👋 Server stopped")
            httpd.shutdown()

if __name__ == "__main__":
    main()
