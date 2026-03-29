import type { DashboardResponse, IncidentsListResponse, IncidentSnapshotResponse, InsightApiResponse, MaintenancesListResponse, MaintenanceSnapshotResponse, Maintenance, TranslationResponse } from './types';

function getApiBase(): string {
  if (typeof window === 'undefined') return 'http://localhost:3000/api';

  // Tauri app: use the deployed web server API
  if ('__TAURI_INTERNALS__' in window) {
    return 'https://vrcpulse.vrcdevs.com/api';
  }

  // Web: localhost dev or same-origin production
  if (window.location.hostname === 'localhost') {
    return 'http://localhost:3000/api';
  }

  return '/api';
}

const API_BASE = getApiBase();

async function fetchApi<T>(path: string): Promise<T> {
  const res = await fetch(`${API_BASE}${path}`);
  if (!res.ok) throw new Error(`API error: ${res.status}`);
  return res.json();
}

export async function getDashboard(range: string = '12h'): Promise<DashboardResponse> {
  return fetchApi(`/metrics/dashboard?range=${range}`);
}

export async function getIncidents(status: string = 'all'): Promise<IncidentsListResponse> {
  return fetchApi(`/incidents?status=${status}`);
}

export async function getIncidentHistory(incidentId: string): Promise<IncidentSnapshotResponse[]> {
  return fetchApi(`/incidents/history/${incidentId}`);
}

export async function getMaintenances(status: string = 'upcoming'): Promise<MaintenancesListResponse> {
  return fetchApi(`/maintenances?status=${status}`);
}

export async function getMaintenanceById(id: string): Promise<Maintenance> {
  return fetchApi(`/maintenances/${id}`);
}

export async function getMaintenanceHistory(maintenanceId: string): Promise<MaintenanceSnapshotResponse[]> {
  return fetchApi(`/maintenances/history/${maintenanceId}`);
}

export async function getInsight(): Promise<InsightApiResponse> {
  return fetchApi('/insights/latest');
}

export async function getTranslation(type: 'incident' | 'maintenance', id: string, locale: string = 'ko'): Promise<TranslationResponse> {
  return fetchApi(`/translate?type=${type}&id=${encodeURIComponent(id)}&locale=${locale}`);
}
