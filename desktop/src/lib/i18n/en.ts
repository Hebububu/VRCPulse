export default {
  // StatusBar
  'status.server': 'VRChat Server Status',
  'status.operational': 'Operational',
  'status.minor': 'Minor Issues',
  'status.major': 'Major Outage',
  'status.critical': 'Critical Outage',
  'status.connecting': 'Connecting...',
  'status.lastUpdated': 'Last updated',

  // Charts
  'chart.onlineUsers': 'Online Users',
  'chart.apiLatency': 'API Latency',
  'chart.apiRequests': 'API Requests',
  'chart.errorRate': 'Error Rate',
  'chart.steamAuth': 'Steam Auth Success',
  'chart.metaAuth': 'Meta Auth Success',
  'chart.platformShare': 'Platform Share',
  'chart.noData': 'No data',
  'chart.hint.apiRequests': 'Normalized API request level relative to average capacity',
  'chart.hint.platformShare': 'Percentage of total authentications by platform (Steam vs Meta/Oculus)',

  // Incidents
  'incidents.recent': 'Recent Incidents',
  'incidents.viewAll': 'View All',
  'incidents.noRecords': 'No incidents recorded',
  'incidents.history': 'Incident History',
  'incidents.updates': 'Updates',
  'incidents.changeHistory': 'Change History',
  'incidents.viewSource': 'View on status.vrchat.com',
  'incidents.noUpdates': 'No updates yet',
  'incidents.notFound': 'Incident not found',
  'incidents.duration': 'Duration',
  'incidents.time': 'Time',
  'incidents.status': 'Status',
  'incidents.impact': 'Impact',

  // Filters
  'filter.all': 'All',
  'filter.resolved': 'Resolved',
  'filter.investigating': 'Investigating',
  'filter.monitoring': 'Monitoring',

  // Promo
  'promo.discord': 'Get Discord Alerts',
  'promo.discordDesc': 'Add bot to your server',
  'promo.desktop': 'Desktop App',
  'promo.desktopDesc': 'System tray + notifications',

  // Update
  'update.available': 'is available',
  'update.now': 'Update Now',
  'update.later': 'Later',
  'update.updating': 'Updating...',

  // Navigation
  'nav.dashboard': '← Dashboard',
  'nav.incidents': '← Incidents',

  // Settings
  'settings.title': 'Settings',
  'settings.language': 'Language',
  'settings.languageDesc': 'Display language',
  'settings.app': 'App Settings',
  'settings.closeToTray': 'Minimize to tray on close',
  'settings.closeToTrayDesc': 'Keep running in system tray when closing the window',
  'settings.notifications': 'Incident notifications',
  'settings.notificationsDesc': 'Show native notification when new incidents are detected',

  // Maintenance
  'maintenance.upcoming': 'Upcoming Maintenance',
  'maintenance.inProgress': 'Maintenance In Progress',
  'maintenance.completed': 'Completed',
  'maintenance.scheduled': 'Scheduled',
  'maintenance.scheduledFor': 'Scheduled for',
  'maintenance.scheduledUntil': 'Until',
  'maintenance.viewAll': 'View All',
  'maintenance.history': 'Maintenance History',
  'maintenance.changeHistory': 'Change History',
  'maintenance.viewSource': 'View on status.vrchat.com',
  'maintenance.bannerText': 'Maintenance in progress',
  'maintenance.bannerLink': 'View details',
  'maintenance.noRecords': 'No scheduled maintenance',
  'maintenance.recent': 'Maintenance',
  'maintenance.notFound': 'Maintenance not found',
  'maintenance.description': 'Description',

  // AI Insight
  'insight.confidence.high': 'High',
  'insight.confidence.medium': 'Medium',
  'insight.confidence.low': 'Low',
  'insight.trustLabel': 'Confidence',
  'insight.nextAnalysis': 'Next analysis',
  'insight.basis': '24-hour basis',
  'insight.ariaLabel': 'AI Server Status Analysis',

  // Error
  'error.retry': 'Retry',
  'error.connectionLost': 'Connection lost',
  'error.failedToLoad': 'Failed to load',
  'error.loading': 'Loading...',
} as const;
