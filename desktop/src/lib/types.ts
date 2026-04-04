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
  id: string;
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
  description: string;
  status: string;
  scheduled_for: string;
  scheduled_until: string;
}

export interface MaintenancesListResponse {
  maintenances: Maintenance[];
}

export interface InsightSummary {
  headline: string;
  bullets: string[];
  affected_surfaces: string[];
  reasoning_basis: string[];
  confidence: number;
  severity: 'stable' | 'warning' | 'critical';
}

export interface AiInsightResponse {
  id: number;
  scope: string;
  trigger_type: string;
  headline: string;
  summary: InsightSummary;
  confidence: number;
  model_id: string;
  created_at: string;
  expires_at: string;
}

export interface InsightBundle {
  en: AiInsightResponse | null;
  ko: AiInsightResponse | null;
  jp: AiInsightResponse | null;
}

export interface InsightApiResponse {
  insight: InsightBundle | null;
}

export interface MaintenanceSnapshotResponse {
  maintenance_id: string;
  title: string;
  status: string;
  scheduled_for: string;
  scheduled_until: string;
  fetched_at: string;
}

export interface TranslatedUpdate {
  update_id: string;
  translated_body: string;
}

export interface TranslationResponse {
  translated_name: string;
  translated_body: string;
  translated_updates: TranslatedUpdate[];
  cached: boolean;
}

export interface ComponentStatus {
  component_id: string;
  name: string;
  current_status: string;
  buckets: string[];
}

export interface IncidentSnapshotResponse {
  incident_id: string;
  title: string;
  impact: string;
  status: string;
  update_count: number;
  fetched_at: string;
}
