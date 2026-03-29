export default {
  // StatusBar
  'status.server': 'VRChat 서버 상태',
  'status.operational': '정상 운영',
  'status.minor': '경미한 문제',
  'status.major': '주요 장애',
  'status.critical': '심각한 장애',
  'status.connecting': '연결 중...',
  'status.lastUpdated': '마지막 업데이트',

  // Charts
  'chart.onlineUsers': '접속자 수',
  'chart.apiLatency': 'API 응답 시간',
  'chart.apiRequests': 'API 요청',
  'chart.errorRate': '오류율',
  'chart.steamAuth': 'Steam 인증 성공률',
  'chart.metaAuth': 'Meta 인증 성공률',
  'chart.platformShare': '플랫폼 비율',
  'chart.noData': '데이터 없음',
  'chart.hint.apiRequests': '평균 용량 대비 현재 API 요청 수준',
  'chart.hint.platformShare': '전체 인증 중 플랫폼별 비율 (Steam vs Meta/Oculus)',

  // Incidents
  'incidents.recent': '최근 인시던트',
  'incidents.viewAll': '전체 보기',
  'incidents.noRecords': '기록된 인시던트가 없습니다',
  'incidents.history': '인시던트 기록',
  'incidents.updates': '업데이트',
  'incidents.changeHistory': '변경 이력',
  'incidents.viewSource': 'status.vrchat.com에서 보기',
  'incidents.noUpdates': '업데이트가 없습니다',
  'incidents.notFound': '인시던트를 찾을 수 없습니다',
  'incidents.duration': '소요 시간',
  'incidents.time': '시간',
  'incidents.status': '상태',
  'incidents.impact': '영향도',

  // Filters
  'filter.all': '전체',
  'filter.resolved': '해결됨',
  'filter.investigating': '조사 중',
  'filter.monitoring': '모니터링 중',

  // Promo
  'promo.discord': 'Discord 알림 받기',
  'promo.discordDesc': '서버에 봇 추가하기',
  'promo.desktop': '데스크탑 앱',
  'promo.desktopDesc': '트레이 아이콘 + 알림',

  // Update
  'update.available': '업데이트 가능',
  'update.now': '지금 업데이트',
  'update.later': '나중에',
  'update.updating': '업데이트 중...',

  // Navigation
  'nav.dashboard': '← 대시보드',
  'nav.incidents': '← 인시던트',

  // Settings
  'settings.title': '설정',
  'settings.language': '언어',
  'settings.languageDesc': '표시 언어',
  'settings.app': '앱 설정',
  'settings.closeToTray': '닫을 때 트레이로 최소화',
  'settings.closeToTrayDesc': '창을 닫을 때 시스템 트레이에서 계속 실행',
  'settings.notifications': '인시던트 알림',
  'settings.notificationsDesc': '새 인시던트 감지 시 네이티브 알림 표시',

  // Maintenance
  'maintenance.upcoming': '예정된 점검',
  'maintenance.inProgress': '점검 진행 중',
  'maintenance.completed': '완료',
  'maintenance.scheduled': '예정',
  'maintenance.scheduledFor': '예정 시간',
  'maintenance.scheduledUntil': '종료 예정',
  'maintenance.viewAll': '전체 보기',
  'maintenance.history': '점검 이력',
  'maintenance.changeHistory': '변경 이력',
  'maintenance.viewSource': 'status.vrchat.com에서 보기',
  'maintenance.bannerText': '현재 점검 중',
  'maintenance.bannerLink': '점검 보러 가기',
  'maintenance.noRecords': '예정된 점검이 없습니다',
  'maintenance.recent': '점검',
  'maintenance.notFound': '점검을 찾을 수 없습니다',
  'maintenance.description': '설명',

  // AI Insight
  'insight.confidence.high': '높음',
  'insight.confidence.medium': '보통',
  'insight.confidence.low': '낮음',
  'insight.trustLabel': '신뢰도',
  'insight.nextAnalysis': '다음 분석',
  'insight.basis': '24시간 기준',
  'insight.ariaLabel': 'AI 서버 상태 분석',

  // Translation
  'translate.button': '한국어로 번역',
  'translate.aiTranslated': 'AI 번역',
  'translate.loading': '번역 중...',
  'translate.error': '번역 실패',
  'translate.showOriginal': '원문 보기',

  // Error
  'error.retry': '재시도',
  'error.connectionLost': '연결 끊김',
  'error.failedToLoad': '로드 실패',
  'error.loading': '로딩 중...',
} as const;
