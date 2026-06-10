# Compact-DSL (C-DSL)

AI 통합을 위한 최소한의 토큰 효율적 스크립팅 언어.

## 시작하기

### 1. Rust 설치

Rust가 없으면 [rustup.rs](https://rustup.rs)에서 설치 파일을 받아 실행하세요.

### 2. 클론 및 빌드

```bash
git clone https://github.com/wow1234djkjsh-art/ai-.git
cd ai-
cargo build --release
```

첫 빌드는 의존성 다운로드로 인해 1~2분 정도 걸립니다.

### 3. 실행

대화형 REPL 실행:

```bash
# Linux / macOS
./target/release/c-dsl

# Windows
.\target\release\c-dsl.exe
```

스크립트 파일 실행:

```bash
# Linux / macOS
./target/release/c-dsl --run script.cdsl

# Windows
.\target\release\c-dsl.exe --run script.cdsl
```

---

## 언어 레퍼런스

### 변수

`=`로 대입. REPL과 스크립트 모두 줄 간 변수 유지.

```
x = 5
y = x * 2
```

### 산술 연산

연산자: `+`, `-`, `*`, `/`, `>`, `<`, `>=`, `<=`, `==`, `!=`. 문자열 연결도 `+`.

```
1 + 2
10 - 3
4 * 5
20 / 4
"hello" + " world"
```

### 비교 연산자

모든 비교 연산자는 `1`(참) 또는 `0`(거짓) 반환.

```
1 == 1          // 1
1 != 2          // 1
3 >= 2          // 1
1 <= 1          // 1
"a" == "a"      // 1
"a" != "b"      // 1
```

- `>=`, `<=`는 숫자만 지원. 문자열에 사용하면 에러 반환.
- `nil` 비교는 항상 에러. nil 체크는 `.type` 필드 사용.
- 타입이 다른 값끼리 `==`/`!=` 비교하면 에러 반환 (예: `1 == "1"` → 에러).

### 논리 연산자

`and`, `or`, `not` — 단락 평가. `1`(참) 또는 `0`(거짓) 반환.

```
1 > 0 and 2 > 0    // 1
0 > 1 or 2 > 0     // 1
not 0               // 1
```

### 함수 정의

`fn <이름> <매개변수> => <본문>`. 매개변수는 쉼표로 구분.

```
fn add a, b => a + b
fn square x => x * x
```

익명(람다) 함수:

```
fn x => x * 2
```

### 함수 호출

괄호 또는 공백으로 호출.

```
add(1, 2)
add 1, 2
square 7
```

### 조건식 (삼항)

`? <조건> : <참> : <거짓>`

```
x = 5
? x > 0 : x : 0
? x > 3 : x * 2 : 0
```

### 파이프 연산자

`<식> | <함수>` — 왼쪽 값을 오른쪽 함수의 첫 번째 인수로 전달.

```
fn double x => x * 2
3 | double
3 | double | double
add 1, 2 | double
```

### each 반복문

`each <항목1>, <항목2>, ... : <함수>` — 모든 항목에 함수 적용, 마지막 결과 반환.

```
each 1, 2, 3 : fn x => x * 2
fn triple x => x * 3
each 10, 20, 30 : triple
```

### 리스트

순서 있는 컬렉션. `[n]`으로 인덱스 접근 (0부터 시작).

```
lst = [1, 2, 3]
lst[0]            // 1
lst[2]            // 3
```

범위 초과 접근 시 런타임 에러 반환.

### 딕셔너리

키-값 컬렉션. 키는 문자열(따옴표 생략 가능). `["키"]` 또는 점 표기법으로 접근.

```
user = {name: "alice", age: 30}
user["name"]      // "alice"
user.name         // "alice"  (점 표기법)
user.age          // 30
```

없는 키는 `nil` 반환.

### 점 표기법 (필드 접근)

`.필드`는 딕셔너리와 에러 값 모두에서 사용 가능.

```
d = {x: 10, y: 20}
d.x               // 10

e = unknown_fn()
e.type            // "error"
e.message         // "unknown function: unknown_fn"
```

### 재귀 함수

함수는 자기 이름으로 자신을 호출할 수 있음.

```
fn fact n => ? n > 0 : n * fact n-1 : 1
fact 5            // 120
```

### 멀티 라인 스크립트

구문은 줄바꿈 또는 `;`으로 구분.

```
fn add a, b => a + b
fn double x => x * 2
result = add 3, 4 | double
result            // 14
```

---

## 에러 처리

### 에러를 값으로 다루기

런타임 에러는 일급 값. 실패한 호출은 크래시 대신 에러 값을 반환.

```
e = unknown_fn()
e.type            // "error"
e.message         // "unknown function: unknown_fn"
```

에러 여부에 따라 분기:

```
result = risky_fn()
? result.type == "error" : result.message : result
```

### try/catch/end

변수에 대입하지 않은 단독 표현식의 에러를 잡을 때 사용.

```
try
  unknown_fn()
catch err
  print(err.message)
end
```

에러가 없으면 try 본문의 결과가 그대로 반환됨.

```
try
  42
catch err
  0
end
// → 42
```

**주의:** `x = bad_fn()`은 에러를 `x`에 저장하고 계속 실행됨 — catch를 트리거하지 않음. 단독 표현식에서 전파되는 에러만 catch가 잡음.

### 에러 메시지 목록

| 상황 | 메시지 |
|------|--------|
| 정의되지 않은 변수 | `undefined variable: foo` |
| 알 수 없는 함수 | `unknown function: pritn` |
| 인수 개수 불일치 | `arity mismatch: fn expects 2 args, got 1` |
| 타입 에러 | `type error: '+' not supported for these types` |
| 0으로 나누기 | `division by zero` |
| 인덱스 범위 초과 | `index out of bounds: 5` |
| 잘못된 인덱스 | `invalid index: must be a non-negative integer` |
| 파싱 에러 | `parse error: ...` |

스크립트 모드(`--run`)에서 처리되지 않은 에러는 stderr에 `Runtime Error: <메시지>`를 출력하고 종료 코드 1로 종료.

---

## 내장 함수

### `print`

값을 stdout에 출력하고 그대로 반환.

```
print 42
print "hello"
add 1, 2 | print
```

### `eval`

현재 스코프에서 C-DSL 표현식 문자열을 평가.

```
eval "2 + 3"
x = 10
eval "x * 3"     // 30
```

### `model`

응답 캐싱 옵션과 함께 언어 모델을 호출.

```
model "model-id" "prompt"
model "model-id" "prompt" "code"
model "model-id" "prompt" "code" "true"
```

| 위치 | 값 | 설명 |
|------|----|------|
| 1 | `"model-id"` | 모델 식별자 |
| 2 | `"prompt"` | 입력 프롬프트 |
| 3 (선택) | `"code"` | 일반 텍스트 대신 C-DSL 표현식 반환 |
| 4 (선택) | `"true"` | 캐시 무시 (강제 재실행) |

응답은 모델 ID, 프롬프트, 포맷으로 만든 SHA-256 키로 `~/.c-dsl/cache/`에 캐싱됨.

`eval`과 조합:

```
result = eval(model "codegen-model" "표현식 작성" "code")
```

---

## 테스트 실행

```bash
cargo test
```
