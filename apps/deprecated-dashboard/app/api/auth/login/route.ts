// Authentication API route (server-side)
// Handles admin authentication without exposing credentials to client
import { NextRequest, NextResponse } from 'next/server';
import axios from 'axios';

const API_URL = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8080';

export async function POST(request: NextRequest) {
  try {
    const body = await request.json();
    const { username, password } = body;

    // Use admin credentials from environment if not provided
    const adminUsername = username || process.env.API_ADMIN_USERNAME || 'admin';
    const adminPassword = password || process.env.API_ADMIN_PASSWORD || '';

    // Authenticate with v3 API
    const response = await axios.post(`${API_URL}/api/v1/auth/login`, {
      username: adminUsername,
      password: adminPassword,
    });

    // Return token to client
    return NextResponse.json({
      token: response.data.token,
      expires_at: response.data.expires_at,
    });
  } catch (error) {
    console.error('Authentication error:', error);
    return NextResponse.json(
      { error: 'Authentication failed' },
      { status: 401 }
    );
  }
}

