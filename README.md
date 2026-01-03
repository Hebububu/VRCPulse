<div align="center">

<img src="./images/VRCPulse.png" alt="VRCPulse Logo" width="200"/>

# VRCPulse

[![Rust](https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Serenity](https://img.shields.io/badge/Serenity-5865F2?style=for-the-badge&logo=discord&logoColor=white)](https://github.com/serenity-rs/serenity)
[![SeaORM](https://img.shields.io/badge/SeaORM-007ACC?style=for-the-badge&logo=sqlite&logoColor=white)](https://www.sea-ql.org/SeaORM/)
[![License](https://img.shields.io/badge/License-MIT-green.svg?style=for-the-badge)](LICENSE)

<br/>

**VRCPulse** is a high-performance Discord bot written in Rust.<br/>
It monitors VRChat server status and provides real-time visualized dashboards.

[![Add to Discord](https://img.shields.io/badge/Add%20to%20Discord-5865F2?style=for-the-badge&logo=discord&logoColor=white)](https://vrcpulse.vrcdevs.com/install)

[Documentation](./docs/README.md) · [Report Bug](https://github.com/hebububu/VRCPulse/issues) · [Request Feature](https://github.com/hebububu/VRCPulse/issues)

</div>

## 🎬 Demo

<div align="center">
<img src="./images/demo/command-status.webp" alt="Status Command Demo" width="600"/>
</div>

## ✨ Features

- **📊 Visualized Dashboard**: Real-time server metrics charts using `plotters`
- **🤖 Automated Monitoring**: Periodic polling from VRChat Status API & CloudFront metrics

## 🚧 Roadmap

- **📢 User-Driven Alert System**: Threshold-based notifications and user reports
- **⚙️ Guild Configuration**: Per-server settings and notification channels

## 🛠 Tech Stack

- **Language**: Rust (Edition 2024)
- **Discord**: Serenity
- **DB/ORM**: SQLite, Sea-ORM
- **Visualization**: Plotters
- **Runtime**: Tokio

## 🚀 Getting Started

### 1. Add Bot to Your Server

[![Add to Discord](https://img.shields.io/badge/Add%20to%20Discord-5865F2?style=for-the-badge&logo=discord&logoColor=white)](https://vrcpulse.vrcdevs.com/install)

### 2. Run Your Own Bot

**Prerequisites**
- Rust (Latest Stable)
- `sea-orm-cli` (`cargo install sea-orm-cli`)

**Installation & Run**
```bash
git clone https://github.com/Hebububu/VRCPulse.git
cd VRCPulse
cp .env.example .env
# Edit .env with your Discord token
sea-orm-cli migrate up
cargo run
```

---

## 🇰🇷 한국어 (Korean)

**VRCPulse**는 VRChat 서버 상태를 실시간으로 모니터링하고 시각화된 데이터를 제공하는 디스코드 봇입니다.

### 주요 기능

- **시각화 대시보드**: 서버 메트릭을 실시간 그래프로 생성합니다
- **자동 모니터링**: VRChat Status API 및 CloudFront 메트릭을 주기적으로 수집합니다

### 개발 예정

- **사용자 기반 알림 시스템**: 임계값 기반 알림 및 사용자 리포트
- **서버별 설정**: 서버별 설정 및 알림 채널 관리

### 시작하기

#### 1. 서버에 봇 추가하기

[![디스코드에 추가](https://img.shields.io/badge/디스코드에%20추가-5865F2?style=for-the-badge&logo=discord&logoColor=white)](https://vrcpulse.vrcdevs.com/install)

#### 2. 직접 봇 실행하기

**필수 조건**
- Rust (최신 안정 버전)
- `sea-orm-cli` (`cargo install sea-orm-cli`)

**설치 및 실행**
```bash
git clone https://github.com/Hebububu/VRCPulse.git
cd VRCPulse
cp .env.example .env
# .env 파일에 Discord 토큰 입력
sea-orm-cli migrate up
cargo run
```

---

## 📄 Documentation

See the [Documentation Index](./docs/README.md) for technical specifications.

## 📜 License

MIT License.
