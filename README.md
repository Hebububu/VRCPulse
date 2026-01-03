<div align="center">

# VRCPulse

[![Rust](https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Serenity](https://img.shields.io/badge/Serenity-5865F2?style=for-the-badge&logo=discord&logoColor=white)](https://github.com/serenity-rs/serenity)
[![SeaORM](https://img.shields.io/badge/SeaORM-007ACC?style=for-the-badge&logo=sqlite&logoColor=white)](https://www.sea-ql.org/SeaORM/)
[![License](https://img.shields.io/badge/License-MIT-green.svg?style=for-the-badge)](LICENSE)

<br/>

**VRCPulse** is a high-performance Discord bot written in Rust.<br/>
It monitors VRChat server status and provides real-time visualized dashboards.

[Documentation](./docs/README.md) · [Report Bug](https://github.com/hebu/vrc-pulse/issues) · [Request Feature](https://github.com/hebu/vrc-pulse/issues)

</div>

## ✨ Features

- **📊 Visualized Dashboard**: Generates real-time server latency charts using `plotters`.
- **🤖 Automated Monitoring**: Periodic polling from VRChat Status API & CloudFront metrics.
- **📢 Smart Alert System**: Instant notifications for official incidents and threshold-based user reports.
- **⚙️ Easy Management**: Simple slash commands (`/config`, `/status`, `/report`).

## 🛠 Tech Stack

- **Language**: Rust (Edition 2024)
- **Discord**: Serenity
- **DB/ORM**: SQLite, Sea-ORM
- **Visualization**: Plotters
- **Runtime**: Tokio

## 🚀 Getting Started

### Prerequisites
- Rust (Latest Stable)
- `sea-orm-cli` (`cargo install sea-orm-cli`)

### Installation & Run
1. `git clone https://github.com/hebu/vrc-pulse.git`
2. Create `.env`:
   ```env
   DISCORD_TOKEN=your_token
   DATABASE_URL=sqlite://data.db?mode=rwc
   ```
3. `cargo run`

---

## 🇰🇷 한국어 (Korean)

**VRCPulse**는 VRChat 서버 상태를 실시간으로 모니터링하고 시각화된 데이터를 제공하는 디스코드 봇입니다.

### 주요 기능
- **시각화 대시보드**: 서버 지연 시간을 그래프로 생성하여 전송합니다.
- **자동 모니터링**: 공식 API 및 메트릭 데이터를 주기적으로 수집합니다.
- **스마트 알림**: 서버 장애 발생 시 설정된 채널로 즉시 알림을 발송합니다.
- **간편한 설정**: 슬래시 명령어를 통해 채널 및 알림 주기를 관리할 수 있습니다.

### 시작하기
상세한 설치 및 실행 방법은 상단의 [Getting Started](#-getting-started) 섹션을 참고해 주세요.

---

## 📄 Documentation
See the [Documentation Index](./docs/README.md) for technical specifications.

## 📜 License
MIT License.
