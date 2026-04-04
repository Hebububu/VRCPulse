export default {
  // StatusBar
  'status.server': 'VRChatサーバー状態',
  'status.operational': '正常稼働',
  'status.minor': '軽微な問題',
  'status.major': '大規模障害',
  'status.critical': '重大障害',
  'status.connecting': '接続中...',
  'status.lastUpdated': '最終更新',

  // Charts
  'chart.onlineUsers': 'オンラインユーザー',
  'chart.apiLatency': 'APIレイテンシー',
  'chart.apiRequests': 'APIリクエスト',
  'chart.errorRate': 'エラー率',
  'chart.steamAuth': 'Steam認証成功率',
  'chart.metaAuth': 'Meta認証成功率',
  'chart.platformShare': 'プラットフォーム比率',
  'chart.noData': 'データなし',
  'chart.hint.apiRequests': '平均容量に対する現在のAPIリクエストレベル',
  'chart.hint.platformShare': '全認証におけるプラットフォーム別比率（Steam vs Meta/Oculus）',

  // Incidents
  'incidents.recent': '最近のインシデント',
  'incidents.viewAll': 'すべて表示',
  'incidents.noRecords': '記録されたインシデントはありません',
  'incidents.history': 'インシデント履歴',
  'incidents.updates': 'アップデート',
  'incidents.changeHistory': '変更履歴',
  'incidents.viewSource': 'status.vrchat.comで表示',
  'incidents.noUpdates': 'アップデートはありません',
  'incidents.notFound': 'インシデントが見つかりません',
  'incidents.duration': '所要時間',
  'incidents.time': '時間',
  'incidents.status': 'ステータス',
  'incidents.impact': '影響度',

  // Filters
  'filter.all': 'すべて',
  'filter.resolved': '解決済み',
  'filter.investigating': '調査中',
  'filter.monitoring': '監視中',

  // Promo
  'promo.discord': 'Discordアラートを受信',
  'promo.discordDesc': 'サーバーにBotを追加',
  'promo.desktop': 'デスクトップアプリ',
  'promo.desktopDesc': 'システムトレイ＋通知',

  // Update
  'update.available': 'が利用可能です',
  'update.now': '今すぐ更新',
  'update.later': '後で',
  'update.updating': '更新中...',

  // Navigation
  'nav.dashboard': '← ダッシュボード',
  'nav.incidents': '← インシデント',

  // Settings
  'settings.title': '設定',
  'settings.language': '言語',
  'settings.languageDesc': '表示言語',
  'settings.app': 'アプリ設定',
  'settings.closeToTray': '閉じる時にトレイに最小化',
  'settings.closeToTrayDesc': 'ウィンドウを閉じてもシステムトレイで実行を継続',
  'settings.notifications': 'インシデント通知',
  'settings.notificationsDesc': '新しいインシデント検出時にネイティブ通知を表示',

  // Maintenance
  'maintenance.upcoming': '予定されたメンテナンス',
  'maintenance.inProgress': 'メンテナンス実施中',
  'maintenance.completed': '完了',
  'maintenance.scheduled': '予定',
  'maintenance.scheduledFor': '予定時間',
  'maintenance.scheduledUntil': '終了予定',
  'maintenance.viewAll': 'すべて表示',
  'maintenance.history': 'メンテナンス履歴',
  'maintenance.changeHistory': '変更履歴',
  'maintenance.viewSource': 'status.vrchat.comで表示',
  'maintenance.bannerText': '現在メンテナンス中',
  'maintenance.bannerLink': '詳細を見る',
  'maintenance.noRecords': '予定されたメンテナンスはありません',
  'maintenance.recent': 'メンテナンス',
  'maintenance.notFound': 'メンテナンスが見つかりません',
  'maintenance.description': '説明',

  // Components
  'component.title': 'コンポーネント',
  'component.all_operational': '全{n}コンポーネント正常',
  'component.operational': '正常',
  'component.degraded': 'パフォーマンス低下',
  'component.partial_outage': '一部障害',
  'component.major_outage': '大規模障害',
  'component.unknown': '不明',
  'component.collapse': 'コンポーネントを折りたたむ',
  'component.expand': 'コンポーネントを展開',
  'component.ago': '前',
  'component.now': '現在',
  'component.name.API / Website': 'API / ウェブサイト',
  'component.name.Realtime Networking': 'リアルタイムネットワーキング',
  'component.name.Authentication / Login': '認証 / ログイン',
  'component.name.Social / Friends List': 'ソーシャル / フレンドリスト',
  'component.name.SDK Asset Uploads': 'Unity SDKアセットアップロード',
  'component.name.Realtime Player State Changes': 'リアルタイムプレイヤー状態変更',
  'component.name.USA, West (San José)': 'アメリカ西部（サンノゼ）',
  'component.name.USA, East (Washington D.C.)': 'アメリカ東部（ワシントンD.C.）',
  'component.name.Europe (Amsterdam)': 'ヨーロッパ（アムステルダム）',
  'component.name.Japan (Tokyo)': '日本（東京）',

  // AI Insight
  'insight.confidence.high': '高',
  'insight.confidence.medium': '中',
  'insight.confidence.low': '低',
  'insight.trustLabel': '信頼度',
  'insight.nextAnalysis': '次の分析',
  'insight.basis': '24時間基準',
  'insight.ariaLabel': 'AIサーバー状態分析',

  // Translation
  'translate.button': '日本語に翻訳',
  'translate.aiTranslated': 'AI翻訳',
  'translate.loading': '翻訳中...',
  'translate.error': '翻訳に失敗しました',
  'translate.showOriginal': '原文を表示',

  // Error
  'error.retry': '再試行',
  'error.connectionLost': '接続が切断されました',
  'error.failedToLoad': '読み込みに失敗しました',
  'error.loading': '読み込み中...',
} as const;
