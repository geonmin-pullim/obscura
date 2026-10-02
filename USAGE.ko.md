# obscura 사용법

obscura는 Chrome 없이 동작하는 헤드리스 브라우저입니다. CDP 서버로 띄워
Puppeteer나 Playwright로 조작하거나, 명령 한 줄로 페이지를 받아 올 수 있습니다.
이 문서는 이 포크를 실제로 쓰는 방법만 정리한 것입니다. 전체 설명은
[README.md](README.md)와 [docs/](docs/README.md)(영문)에 있습니다.

## 1. 빌드

필요한 것: Rust 1.75 이상([rustup.rs](https://rustup.rs)), C 컴파일러, CMake, Clang(libclang),
디스크 여유 공간 약 5 GB. 첫 빌드는 몇 분 걸리고, 이후에는 금방 끝납니다.

```sh
# 렌더링 없음 + 스텔스 (평소 쓰는 빌드)
CARGO_TARGET_DIR=target-norender \
  cargo build --release -p obscura-cli --bins --no-default-features --features stealth
```

실행 파일은 `target-norender/release/obscura`에 생깁니다.

| 빌드 | 명령의 feature 부분 | 차이 |
|---|---|---|
| 렌더링 없음 + 스텔스 | `--no-default-features --features stealth` | 가볍고 빠름. 레이아웃, 스크린샷, PDF 없음 |
| 렌더링 + 스텔스 | `--features render,stealth` | 레이아웃, 스크린샷, PDF 포함. 실행 파일은 `target/release/obscura` |

`stealth` feature가 있어야 Chrome의 TLS 핑거프린트 흉내와 트래커 차단이 들어갑니다.
봇 차단이 있는 사이트에는 이 feature로 빌드한 것을 쓰세요.

## 2. CDP 서버로 띄우기

```sh
target-norender/release/obscura --stealth serve --port 9225
```

- `--stealth`: 스텔스 모드. 일관된 브라우저 핑거프린트를 씁니다.
- `--port`: CDP 포트(기본 9222). `127.0.0.1`에서만 받습니다.
- 터미널을 닫으면 꺼집니다. 켜 둔 채로 다른 터미널에서 접속합니다.

자주 쓰는 옵션:

| 옵션 | 뜻 |
|---|---|
| `--proxy http://호스트:포트` | 모든 요청을 프록시로 보냄 |
| `--storage-dir <폴더>` | 쿠키와 저장소를 폴더에 보관해 다음 실행에도 유지 |
| `--allow-private-network` | `localhost`, 사설 IP 접속 허용(기본은 차단). 로컬 테스트 페이지용 |
| `--quiet` | 로그 끄기 |
| `--workers <N>` | 워커 수(기본 1) |

## 3. 지역 설정

접속하는 IP의 지역과 브라우저가 알리는 지역이 같아야 합니다. 환경변수로 지정합니다.

| 환경변수 | 기본값 | 뜻 |
|---|---|---|
| `OBSCURA_TIMEZONE` | `Asia/Seoul` | 시간대. `Date`와 `Intl`이 같은 값을 냄 |
| `OBSCURA_LANGUAGES` | `en-US` | 언어 목록. `navigator.languages`와 `Accept-Language`에 반영 |
| `OBSCURA_GEOLOCATION` | 고정 기본값 | `navigator.geolocation` 좌표, `위도,경도` |

한국에서 쓸 때:

```sh
OBSCURA_LANGUAGES=ko-KR,ko,en-US,en \
OBSCURA_GEOLOCATION=37.5665,126.9780 \
  target-norender/release/obscura --stealth serve --port 9225
```

시간대는 기본이 한국이라 따로 지정하지 않아도 됩니다. 다른 지역 프록시를 쓸 때는
세 값을 모두 그 지역에 맞춥니다(예: `OBSCURA_TIMEZONE=America/New_York`).

## 4. Puppeteer로 접속하기

```sh
npm install puppeteer-core
```

```js
import puppeteer from 'puppeteer-core';

const browser = await puppeteer.connect({ browserURL: 'http://127.0.0.1:9225', defaultViewport: null });
const ctx = await browser.createBrowserContext();   // 쿠키가 분리된 새 세션
const page = await ctx.newPage();

const res = await page.goto('https://example.com/', { waitUntil: 'domcontentloaded' });
console.log(res.status(), await page.title());

const text = await page.evaluate(() => document.body.innerText);   // 페이지 안에서 실행
await page.setCookie({ name: 'a', value: '1', domain: '.example.com', path: '/' });

await ctx.close();
await browser.disconnect();   // close()가 아니라 disconnect(): obscura는 계속 켜 둠
```

- `createBrowserContext()`마다 쿠키가 따로 관리됩니다. 새 방문자처럼 시작하려면 새 컨텍스트를 만듭니다.
- 렌더링 없는 빌드에는 실제 레이아웃이 없습니다. 마우스 좌표 클릭은 다른 요소에 닿을 수 있으니, 요소에 포커스를 주고 입력하거나 `page.evaluate`로 다루세요.
- Playwright는 `chromium.connectOverCDP('ws://127.0.0.1:9225')`로 접속합니다.

## 5. 서버 없이 한 번만 받아 오기

```sh
obscura --stealth fetch https://example.com --dump text        # 본문 텍스트
obscura --stealth fetch https://example.com --dump html        # 렌더링된 HTML
obscura --stealth fetch https://example.com --dump markdown    # 마크다운
obscura --stealth fetch https://example.com --dump cookies     # 쿠키(HttpOnly 포함)
```

`--dump` 값: `html`, `text`, `links`, `markdown`, `original`(응답 원본), `assets`, `cookies`.
`--selector`로 특정 요소만, `--timeout`(초, 기본 30)으로 제한 시간을 정합니다.

여러 URL을 한꺼번에:

```sh
obscura --stealth scrape https://a.example https://b.example --concurrency 10 --format json
```

## 6. 문제가 생겼을 때

| 증상 | 조치 |
|---|---|
| 접속 시 `ECONNREFUSED 127.0.0.1:<포트>` | obscura가 꺼져 있거나 포트가 다릅니다. 2번을 다시 실행합니다 |
| `operation timed out` | 프록시가 응답하지 않습니다. `curl -x <프록시> https://api.ipify.org`로 확인합니다 |
| `localhost` 페이지가 열리지 않음 | `--allow-private-network`를 붙입니다 |
| 시간대나 언어가 다르게 나옴 | 3번의 환경변수를 확인합니다. 호스트에 `TZ`가 설정돼 있으면 그 값이 쓰입니다 |
| 로그가 너무 많음 | `--quiet`를 붙이거나 `RUST_LOG=error`로 실행합니다 |

더 자세한 내용(영문): [CLI 전체 옵션](docs/CLI-reference.md),
[환경변수](docs/Environment-variables.md),
[스텔스와 프록시](docs/Configure-stealth-and-proxies.md),
[Puppeteer](docs/Use-with-Puppeteer.md), [Playwright](docs/Use-with-Playwright.md).
