// API Proxy Route
// Proxies requests to v3 API with authentication
import { NextRequest, NextResponse } from 'next/server';
import axios, { AxiosError } from 'axios';

const API_URL = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8080';

export async function GET(
  request: NextRequest,
  { params }: { params: Promise<{ path: string[] }> }
) {
  const { path } = await params;
  return handleProxyRequest(request, path, 'GET');
}

export async function POST(
  request: NextRequest,
  { params }: { params: Promise<{ path: string[] }> }
) {
  const { path } = await params;
  return handleProxyRequest(request, path, 'POST');
}

export async function PUT(
  request: NextRequest,
  { params }: { params: Promise<{ path: string[] }> }
) {
  const { path } = await params;
  return handleProxyRequest(request, path, 'PUT');
}

export async function DELETE(
  request: NextRequest,
  { params }: { params: Promise<{ path: string[] }> }
) {
  const { path } = await params;
  return handleProxyRequest(request, path, 'DELETE');
}

async function handleProxyRequest(
  request: NextRequest,
  pathSegments: string[],
  method: string
) {
  try {
    const url = new URL(request.url);
    const searchParams = url.searchParams.toString();
    const path = `/${pathSegments.join('/')}${searchParams ? `?${searchParams}` : ''}`;
    const targetUrl = `${API_URL}${path}`;

    // Get auth token from request headers
    const authHeader = request.headers.get('authorization');

    const config: {
      method: string;
      url: string;
      headers: Record<string, string>;
      data?: unknown;
    } = {
      method,
      url: targetUrl,
      headers: {
        'Content-Type': 'application/json',
      },
    };

    if (authHeader) {
      config.headers.Authorization = authHeader;
    }

    if (method === 'POST' || method === 'PUT') {
      const body = await request.json().catch(() => ({}));
      config.data = body;
    }

    const response = await axios(config);

    return NextResponse.json(response.data, {
      status: response.status,
      headers: {
        'Access-Control-Allow-Origin': '*',
        'Access-Control-Allow-Methods': 'GET, POST, PUT, DELETE, OPTIONS',
        'Access-Control-Allow-Headers': 'Content-Type, Authorization',
      },
    });
  } catch (error) {
    const axiosError = error as AxiosError;
    return NextResponse.json(
      {
        error: axiosError.response?.data || axiosError.message,
      },
      {
        status: axiosError.response?.status || 500,
      }
    );
  }
}

