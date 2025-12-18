# HDDLLFRT - HDD 로우 레벨 포맷 Rust(ool)

HDDGURU의 HDD LLF Low Level Format Tool에서 영감을 받아 Rust로 작성된 크로스 플랫폼 저장 장치 로우 레벨 포맷 도구입니다.

## 기능

🔧 **핵심 작업:**

- **로우 레벨 포맷 (Low Level Format)**: 전체 장치를 0으로 채움 (Zero-fill)
- **빠른 포맷 (Quick Format)**: 시작과 끝 섹터만 빠르게 포맷
- **보안 삭제 (Secure Erase)**: 다양한 패턴으로 여러 번 덮어쓰기
- **장치 검증 (Device Verification)**: 모든 섹터 읽기 및 검증
- **S.M.A.R.T. 데이터**: 장치 상태 정보 확인

🖥️ **크로스 플랫폼 지원:**

<!-- - ✅ Linux (Ubuntu, Fedora, Arch에서 테스트됨) -->
- ✅ Windows (Windows 10/11)
<!-- - ✅ macOS (10.15+) -->

⚡ **특징:**

- 대화형 CLI 메뉴 시스템
- 실시간 진행 상황 추적
- 다중 안전 확인 절차
- 자동 마운트 감지
- 관리자 권한 확인
- 포괄적인 오류 처리

## 설치

### 필수 조건

- Rust 1.70 이상
- 장치 작업을 위한 관리자(Administrator)/루트(root) 권한

### 소스에서 빌드하기

```bash
# 저장소 복제
git clone https://github.com/DevNergis/HDDLLFRT.git
cd HDDLLFRT

# 릴리스 버전 빌드
cargo build --release

# 바이너리는 target/release/hddllfrt (Windows의 경우 hddllfrt.exe)에 위치합니다.
```

### 빠른 빌드 명령어

```bash
# 디버그 빌드 (컴파일 속도는 빠르지만 실행 속도는 느림)
cargo build

# 릴리스 빌드 (성능 최적화)
cargo build --release

# 별도 빌드 없이 바로 실행
cargo run --release
```

## 사용법

### 도구 실행

**Linux / macOS:**

```bash
sudo ./target/release/hddllfrt
```

**Windows:**

```cmd
# 관리자 권한으로 실행
.\target\release\hddllfrt.exe
```

### 대화형 메뉴

이 도구는 다음과 같은 옵션이 있는 대화형 메뉴를 제공합니다:

1. **장치 목록 (List Devices)** - 감지된 모든 저장 장치 표시
2. **로우 레벨 포맷 (Low Level Format)** - 전체 제로 필(zero-fill) 포맷 수행
3. **빠른 포맷 (Quick Format)** - 빠른 포맷 (시작과 끝 부분만)
4. **보안 삭제 (Secure Erase)** - 다중 패스 보안 삭제 (1-10 패스)
5. **장치 검증 (Verify Device)** - 모든 섹터 읽기 및 검증
6. **S.M.A.R.T. 데이터 보기 (View S.M.A.R.T. Data)** - 장치 상태 정보 표시
7. **종료 (Exit)** - 애플리케이션 종료

### 안전 기능

이 도구에는 여러 안전 메커니즘이 포함되어 있습니다:

- ⚠️ **관리자 확인**: 관리자 권한 필요
- ⚠️ **마운트 감지**: 마운트된 장치의 포맷 방지
- ⚠️ **다중 확인**: 파괴적인 작업에 대한 2단계 확인 절차
- ⚠️ **명확한 경고**: 데이터 손실에 대한 눈에 띄는 경고

## 경고

⚠️ **중요 경고** ⚠️

이 도구는 저장 장치에 대해 **파괴적인** 작업을 수행합니다. 장치를 포맷하면:

- **모든 데이터가 영구적으로 삭제됩니다**
- **이 작업은 되돌릴 수 없습니다**
- **데이터 복구가 불가능합니다**

진행하기 전에 항상 올바른 장치를 선택했는지 확인하십시오!

<!-- ## 기술적 세부 사항

### 장치 감지

- **Linux**: `/sys/block` 및 `/dev`를 사용하여 장치 열거
- **Windows**: Windows API를 사용하여 물리적 드라이브(Physical Drives)에 액세스
- **macOS**: `diskutil` 및 IOKit을 사용하여 장치 열거

### 로우 레벨 포맷 프로세스

로우 레벨 포맷 작업:

1. 배타적 쓰기 권한으로 장치 열기
2. 1MB 청크 단위로 전체 장치에 0 쓰기
3. 진행 상황 추적 및 실시간 피드백 제공
4. 모든 데이터 동기화(Sync)로 쓰기 완료 보장
5. 작업 완료 검증

### 보안 삭제 (Secure Erase)

보안 삭제는 다양한 패턴으로 여러 패스를 사용합니다:

- 패스 1: 0x00 (모두 0)
- 패스 2: 0xFF (모두 1)
- 패스 3: 0xAA (교차 패턴)
- 패스 4: 0x55 (역교차 패턴)

추가 패스에 대해 패턴이 반복됩니다.

## 개발

### 프로젝트 구조

```
HDDLLFRT/
├── src/
│   ├── main.rs              # 애플리케이션 진입점
│   ├── device/
│   │   ├── mod.rs           # 장치 데이터 구조
│   │   └── operations.rs    # 포맷/삭제 작업
│   ├── platform/
│   │   ├── mod.rs           # 플랫폼 추상화
│   │   ├── linux.rs         # Linux 구현
│   │   ├── windows.rs       # Windows 구현
│   │   └── macos.rs         # macOS 구현
│   └── ui/
│       └── mod.rs           # 사용자 인터페이스
├── Cargo.toml               # 의존성 및 메타데이터
└── README.md               # 이 파일
```

### 테스트 실행

```bash
cargo test
```

### 코드 스타일

```bash
# 코드 포맷팅
cargo fmt

# 문제 확인
cargo clippy
```

## 의존성

주요 의존성:

- `clap` - 명령줄 인수 파싱
- `dialoguer` - 대화형 CLI 프롬프트
- `indicatif` - 진행률 표시줄
- `console` - 터미널 스타일링
- `anyhow` - 오류 처리
- `sysinfo` - 시스템 정보
- 플랫폼별: `nix` (Linux), `windows` (Windows), `core-foundation` & `io-kit-sys` (macOS)

## 한계

- S.M.A.R.T. 데이터 읽기는 기본적입니다 (완벽한 지원을 위해서는 ATA 명령 구현 필요)
- 일부 USB 장치가 올바르게 감지되지 않을 수 있습니다
- NVMe 관련 기능이 완전히 구현되지 않았습니다
- 모든 작업에 관리자 권한이 필요합니다 -->

## 향후 개선 사항

- [ ] ATA 명령을 통한 전체 S.M.A.R.T. 속성 읽기
- [ ] 불량 섹터 리매핑
- [ ] 벤치마크/속도 테스트
- [ ] 더 많은 장치 유형 지원
- [ ] GUI 버전
- [ ] 구성 파일 지원
- [ ] 파일 로깅
- [ ] 중단된 작업 재개

## 라이선스

AGPL-3.0 license - 자세한 내용은 LICENSE 파일을 참조하십시오.

## 기여

기여는 언제나 환영합니다! 풀 리퀘스트(Pull Request)를 제출하거나 이슈(Issue)를 열어주세요.

## 면책 조항

이 소프트웨어는 어떠한 종류의 보증 없이 "있는 그대로" 제공됩니다. 저자는 이 소프트웨어의 사용으로 인한 데이터 손실이나 손해에 대해 책임을 지지 않습니다. 디스크 작업을 수행하기 전에 항상 데이터를 백업하십시오.

## 크레딧

HDDGURU의 HDD LLF Low Level Format Tool에서 영감을 받았습니다.
DevNergis가 Rust로 개발했습니다.
