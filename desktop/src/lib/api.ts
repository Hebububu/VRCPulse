import type { DashboardResponse, IncidentsListResponse, MaintenancesListResponse } from './types';

const isTauri = typeof window !== 'undefined' && '__TAURI__' in window;

const API_BASE = 'http://localhost:3000/api';

async function fetchApi<T>(path: string): Promise<T> {
  if (isTauri) {
    const { invoke } = await import('@tauri-apps/api/core');
    // For now, fall back to HTTP even in Tauri (commands will be wired later)
    // return invoke(command, args);
  }

  const res = await fetch(`${API_BASE}${path}`);
  if (!res.ok) throw new Error(`API error: ${res.status}`);
  return res.json();
}

export async function getDashboard(range: string = '12h'): Promise<DashboardResponse> {
  return fetchApi(`/metrics/dashboard?range=${range}`);
}

export async function getIncidents(status: string = 'active'): Promise<IncidentsListResponse> {
  return fetchApi(`/incidents?status=${status}`);
}

export async function getMaintenances(status: string = 'upcoming'): Promise<MaintenancesListResponse> {
  return fetchApi(`/maintenances?status=${status}`);
}
