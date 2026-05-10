# HDDLLFRT - HDD Low Level Format Rust(ool)

[English](#english) | [한국어](#한국어)

---

<a name="english"></a>

## English

A cross-platform storage device low-level format utility written in Rust, inspired by HDDGURU's HDD LLF Low Level Format Tool.

### Features

**Core Operations:**

- **Low Level Format**: Zero-fill the entire device
- **Quick Format**: Quickly format only the beginning and end sectors
- **Secure Erase**: Multiple-pass overwrite with different patterns
- **Device Verification**: Read and verify all sectors

**Cross-platform Support:**

- Windows (Windows 10/11)
- Linux
- macOS

**Highlights:**

- Interactive CLI menu system
- Real-time progress tracking
- Multiple safety confirmation steps
- Automatic mount detection
- Exclusive device locking (flock on Unix, volume lock on Windows)
- Administrator privilege verification
- Comprehensive error handling

### Prerequisites

- Rust 1.70+
- Administrator/Root privileges for device operations

### Build from Source

```bash
git clone https://github.com/DevNergis/HDDLLFRT.git
cd HDDLLFRT
cargo build --release
```

### Usage

**Linux / macOS:**
```bash
sudo ./target/release/hddllfrt
```

**Windows (Run as Administrator):**
```cmd
.\target\release\hddllfrt.exe
```

### Safety Features

- Admin privilege check required
- Mount detection prevents formatting mounted devices
- Exclusive locking prevents concurrent access
- Two-step confirmation for destructive operations
- Prominent data loss warnings

### Warning

**This tool performs DESTRUCTIVE operations on storage devices.**

- **All data will be permanently erased**
- **This action CANNOT be undone**
- **Data recovery is impossible**

Always verify you have selected the correct device before proceeding!

### License

AGPL-3.0-or-later - see LICENSE file for details.

---

<a name="한국어"></a>

## 한국어

HDDGURU의 HDD LLF Low Level Format Tool에서 영감을 받아 Rust로 작성된 크로스 플랫폼 저장 장치 로우 레벨 포맷 도구입니다.

### 기능

**핵심 작업:**

- **로우 레벨 포맷 (Low Level Format)**: 전체 장치를 0으로 채움 (Zero-fill)
- **빠른 포맷 (Quick Format)**: 시작과 끝 섹터만 빠르게 포맷
- **보안 삭제 (Secure Erase)**: 다양한 패턴으로 여러 번 덮어쓰기
- **장치 검증 (Device Verification)**: 모든 섹터 읽기 및 검증

**크로스 플랫폼 지원:**

- Windows (Windows 10/11)
- Linux
- macOS

**특징:**

- 대화형 CLI 메뉴 시스템
- 실시간 진행 상황 추적
- 다중 안전 확인 절차
- 자동 마운트 감지
- 배타적 장치 잠금 (Unix: flock, Windows: 볼륨 잠금)
- 관리자 권한 확인
- 포괄적인 오류 처리

### 필수 조건

- Rust 1.70 이상
- 장치 작업을 위한 관리자(Administrator)/루트(root) 권한

### 소스에서 빌드하기

```bash
git clone https://github.com/DevNergis/HDDLLFRT.git
cd HDDLLFRT
cargo build --release
```

### 사용법

**Linux / macOS:**
```bash
sudo ./target/release/hddllfrt
```

**Windows (관리자 권한으로 실행):**
```cmd
.\target\release\hddllfrt.exe
```

### 안전 기능

- 관리자 권한 필요 확인
- 마운트된 장치 포맷 방지
- 배타적 잠금으로 동시 접근 방지
- 파괴적 작업에 대한 2단계 확인 절차
- 데이터 손실에 대한 명확한 경고

### 경고

**이 도구는 저장 장치에 대해 파괴적인 작업을 수행합니다.**

- **모든 데이터가 영구적으로 삭제됩니다**
- **이 작업은 되돌릴 수 없습니다**
- **데이터 복구가 불가능합니다**

진행하기 전에 항상 올바른 장치를 선택했는지 확인하십시오!

### 라이선스

AGPL-3.0-or-later - 자세한 내용은 LICENSE 파일을 참조하십시오.

### 기여

기여는 언제나 환영합니다! 풀 리퀘스트(Pull Request)를 제출하거나 이슈(Issue)를 열어주세요.

### 크레딧

HDDGURU의 HDD LLF Low Level Format Tool에서 영감을 받았습니다.

DevNergis가 Rust로 개발했습니다.
