// Server-side API client for use in Server Components
// Calls API directly without proxy

const API_URL = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8080';

async function apiRequest<T>(
  method: string,
  url: string,
  data?: unknown
): Promise<T> {
  const headers: Record<string, string> = {
    'Content-Type': 'application/json',
  };

  // Note: The v3 API currently doesn't require authentication for most endpoints
  // If authentication is needed in the future, uncomment and configure:
  // const token = await getAuthToken();
  // if (token) {
  //   headers.Authorization = `Bearer ${token}`;
  // }

  const config: RequestInit = {
    method,
    headers,
  };

  if (data && (method === 'POST' || method === 'PUT')) {
    config.body = JSON.stringify(data);
  }

  const response = await fetch(`${API_URL}${url}`, config);

  if (!response.ok) {
    const errorText = await response.text().catch(() => response.statusText);
    throw new Error(`API request failed: ${response.status} ${errorText}`);
  }

  return response.json();
}

export const serverApi = {
  async get<T>(url: string): Promise<T> {
    return apiRequest<T>('GET', url);
  },

  async post<T>(url: string, data?: unknown): Promise<T> {
    return apiRequest<T>('POST', url, data);
  },

  async put<T>(url: string, data?: unknown): Promise<T> {
    return apiRequest<T>('PUT', url, data);
  },

  async delete<T>(url: string): Promise<T> {
    return apiRequest<T>('DELETE', url);
  },
};

