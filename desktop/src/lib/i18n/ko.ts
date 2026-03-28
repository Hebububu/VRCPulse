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

  // Error
  'error.retry': '재시도',
  'error.connectionLost': '연결 끊김',
  'error.failedToLoad': '로드 실패',
  'error.loading': '로딩 중...',
} as const;
