export interface StatusResponse {
  indicator: string;
  description: string;
  updated_at: string;
}

export interface MetricResponse {
  name: string;
  unit: string;
  timestamps: string[];
  values: number[];
}

export interface DashboardResponse {
  metrics: Record<string, MetricResponse>;
  status: StatusResponse;
}

export interface IncidentUpdate {
  status: string;
  body: string;
  created_at: string;
}

export interface Incident {
  id: string;
  name: string;
  status: string;
  impact: string;
  created_at: string;
  updates: IncidentUpdate[];
}

export interface IncidentsListResponse {
  incidents: Incident[];
}

export interface Maintenance {
  id: string;
  name: string;
  status: string;
  scheduled_for: string;
  scheduled_until: string;
}

export interface MaintenancesListResponse {
  maintenances: Maintenance[];
}

export interface IncidentSnapshotResponse {
  incident_id: string;
  title: string;
  impact: string;
  status: string;
  update_count: number;
  fetched_at: string;
}
