# C-DSL 문법 레퍼런스

AI 자동화용 경량 스크립팅 언어. Rust 트리-워킹 인터프리터.  
구문 구분자: `;` 또는 줄바꿈 — 둘 다 가능.

---

## 타입

| 타입 | 리터럴 예시 | 설명 |
|------|-------------|------|
| `Number` | `42`, `3.14`, `-1` | 64비트 부동소수점 |
| `String` | `"hello"` | UTF-8 문자열 |
| `List` | `[1, 2, 3]` | 가변 리스트 |
| `Dict` | `{key: "val", n: 42}` | 순서 있는 키-값 쌍 |
| `Function` | `fn x => x*2` | 클로저 포함 함수 |
| `Nil` | `nil` | 빈 값 |
| `Error` | 런타임 오류 값 | `is_error`, `try/catch`로 검사 |

---

## 전역 상수

| 이름 | 값 |
|------|----|
| `nil` | Nil |
| `true` | `1` |
| `false` | `0` |
| `pi` | `3.141592653589793` |
| `e` | `2.718281828459045` |
| `inf` | 양의 무한대 |
| `nan` | Not-a-Number |

**NaN / Inf 동작:**
- 수학 도메인 오류(`sqrt(-1)`, `log(0)`, `asin(2)` 등)는 Error 대신 `nan` 또는 `inf`를 반환합니다.
- `nan`은 truthy로 판정됩니다 (`nan != 0`이 true이므로).
- `nan == nan` → `0` (IEEE 754 표준; NaN은 자기 자신과 같지 않음).
- NaN 감지: `x != x`가 true이면 NaN입니다 (NaN만이 자기 자신과 같지 않은 유일한 값).

---

## 변수

```
x = 5
y = "hello"
z = x + 3
```

변수에 오류 값을 대입할 수 있습니다 — 오류가 즉시 전파되지 않고 변수에 저장됩니다.

```
result = num "bad"    # result에 Error 저장, 즉시 중단되지 않음
```

---

## 연산자

### 산술

```
x + y    # 덧셈 (문자열 연결도 가능: "a" + "b" → "ab")
x - y    # 뺄셈
x * y    # 곱셈
x / y    # 나눗셈 (0 나누기 → Error)
x % y    # 나머지  (0 나머지 → Error)
x ** y   # 거듭제곱 (우결합: 2 ** 3 ** 2 → 512)
-x       # 단항 음수
```

### 비교

`1`(참) 또는 `0`(거짓) 반환. `>=`, `<=`는 숫자 전용.

```
x > y    x < y    x >= y    x <= y    x == y    x != y
```

`==` / `!=`는 Number, String, Nil 간 비교 가능. 타입 불일치 시 Error.

### 논리

```
x and y      # 둘 다 참이면 1, 단락 평가
x or y       # 하나라도 참이면 1, 단락 평가
not x        # 참/거짓 반전
```

우선순위: `not` > `and` > `or`

### 연산자 우선순위 (높음 → 낮음)

```
[] (인덱스) . (필드 접근)  →  -x (단항)  →  **  →  * / %  →  + -  →  > < >= <=  →  == !=  →  not  →  and  →  or  →  |
```

---

## 참/거짓 판정

| 값 | 판정 |
|----|------|
| `0` | 거짓 |
| `nil` | 거짓 |
| `""` (빈 문자열) | 거짓 |
| `[]` (빈 리스트) | 거짓 |
| `{}` (빈 딕트) | 거짓 |
| Error | 거짓 |
| 그 외 모든 값 | 참 |

---

## 조건문 (3항)

```
?조건 : 참_표현식 : 거짓_표현식
```

```
?x > 0 : x : -x
?x > 10 : "big" : "small"
```

중첩:

```
?x > 10 : "big" : ?x > 0 : "mid" : "small"
```

---

## while 루프

```
while 조건
  본문
end
```

```
i = 0
while i < 5
  print i
  i = i + 1
end
```

### break — 루프 종료

```
i = 0
while i < 10
  i = i + 1
  ?i == 3 : break : nil
end
# i → 3
```

### continue — 다음 반복으로 건너뜀

```
s = 0
i = 0
while i < 5
  i = i + 1
  ?i == 3 : continue : nil   # i==3일 때 s += i 건너뜀
  s = s + i
end
# s → 12 (1+2+4+5)
```

---

## 함수

### 정의

```
fn add a,b => a + b
fn double x => x * 2
fn greet name => "안녕, " + name + "!"
```

파라미터가 없는 함수:

```
fn hello => print "hi"
hello()
```

### 호출

공백 구문:

```
add 1,2
double 5
```

괄호 구문:

```
add(1, 2)
double(5)
```

> **주의 — 음수 첫 번째 인자:** 음수를 첫 번째 인자로 사용할 때는 괄호 구문을 사용하세요: `clamp(-5, 0, 100)` (공백 구문은 `-`를 이항 연산자로 해석합니다).

### return — 함수에서 조기 반환

함수 본문은 단일 표현식입니다. `return`은 그 표현식 안에서 사용할 수 있습니다.

```
fn safe_div a,b => ?b == 0 : return error "0으로 나눌 수 없음" : a/b
safe_div 10,2    # → 5
safe_div 10,0    # → Error

fn classify n => ?n > 0 : return "양수" : ?n < 0 : return "음수" : return "영"
classify 5    # → "양수"
classify 0    # → "영"
```

값 없이 `return` 사용 시 `nil` 반환:

```
fn nothing => return
nothing()    # → nil
```

### 익명 람다

```
fn x => x * 2
fn a,b => a + b
```

람다는 변수에 저장하거나 고차 함수에 직접 전달할 수 있습니다:

```
double = fn x => x * 2
map [1,2,3] fn x => x * 2
```

### 클로저 (외부 변수 캡처)

```
base = 10
fn add_base n => n + base
add_base 5    # → 15
```

### 재귀

```
fn fact n => ?n > 0 : n * fact(n-1) : 1
fact 10    # → 3628800
```

---

## 파이프

왼쪽 결과가 오른쪽 함수의 **첫 번째 인자**로 전달됩니다.

```
3 | double              # → 6
3 | double | double     # → 12
[1,2,3] | sum           # → 6
5 | print               # 5 출력, 5 반환
```

파이프 오른쪽에 추가 인자를 지정할 수 있습니다 (첫 번째 자리에 삽입됨):

```
"hello world" | split " "     # → ["hello", "world"]
[1,2,3] | map fn x => x*2    # → [2, 4, 6]
```

---

## each (반복)

함수를 각 항목에 적용하고, 마지막 결과를 반환합니다.  
`:` 구분자는 필수 문법입니다 (생략 불가).

```
each 1,2,3 : fn x => x * 2       # → 6 (마지막 요소 결과)
each 1,2,3 : fn x => print x     # 1, 2, 3 각각 출력
```

리스트 변수에도 사용 가능:

```
nums = [10, 20, 30]
each nums : fn x => print x
```

이름 있는 함수 사용:

```
fn triple x => x * 3
each 10,20,30 : triple
```

---

## 리스트

```
lst = [1, 2, 3]
lst[0]      # → 1 (0-인덱스)
lst[2]      # → 3
lst[-1]     # → 3 (마지막 요소, 음수 인덱스)
lst[-2]     # → 2 (뒤에서 두 번째)
```

범위 초과 인덱스 → Error.

---

## 딕트

키는 식별자 또는 문자열 리터럴 모두 사용 가능합니다.

```
d = {name: "Alice", age: 30}
d["name"]        # → "Alice"
d.name           # → "Alice" (점 접근)
d["없는키"]      # → nil
```

문자열 키 리터럴 형식:

```
d = {"name": "Alice", "age": 30}
```

### 에러 필드 접근

```
err = error "뭔가 잘못됨"
err.message    # → "뭔가 잘못됨"
err.type       # → "error"
```

---

## try / catch

```
try
  위험한_코드
catch e
  에러 처리 (e는 Error 값)
end
```

```
try
  n = num "abc"
  print n
catch e
  print "오류:" + e.message
end
```

에러 값은 `catch` 블록에서 `.message`, `.type` 필드로 접근할 수 있습니다.

---

## 실행

```powershell
# REPL
./c-dsl

# 스크립트 실행
./c-dsl --run script.dsl

# 인자 전달 (args 리스트로 접근)
./c-dsl --run script.dsl foo bar
# args[0] → "foo", args[1] → "bar"
```

---

## 내장 함수

### 코어

| 함수 | 시그니처 | 설명 |
|------|----------|------|
| `print` | `print v ...` | 값 출력, 마지막 인자 반환. 다중 인자 공백으로 구분 |
| `eval` | `eval "코드"` | C-DSL 문자열 평가 |
| `input` | `input "프롬프트"` | stdin 한 줄 읽기 |
| `env` | `env "VAR"` | 환경변수 읽기, 없으면 nil |

### AI

| 함수 | 시그니처 | 설명 |
|------|----------|------|
| `model` | `model "모델ID" "프롬프트"` | AI 모델 호출, SHA-256 디스크 캐시 |
| `model` | `model "id" "prompt" "code"` | 세 번째 인자로 리터럴 `"code"` 전달 시 별도 캐시 네임스페이스 사용 (API 요청 자체는 동일) |
| `model` | `model "id" "prompt" "" "true"` | 캐시 무시하고 강제 호출 |

```
reply = model "claude-sonnet-4-6" "한국의 수도는?"
print reply
```

### 타입 변환 / 검사

| 함수 | 시그니처 | 설명 |
|------|----------|------|
| `str` | `str v` | 값 → 문자열 |
| `num` | `num s` | 문자열 → 숫자 (실패 시 Error) |
| `type` | `type v` | 타입 이름 반환: `"number"` `"string"` `"list"` `"dict"` `"function"` `"nil"` `"error"` |
| `is_nil` | `is_nil v` | nil이면 1, 아니면 0 |

### 에러 처리

| 함수 | 시그니처 | 설명 |
|------|----------|------|
| `error` | `error "메시지"` | Error 값 생성 |
| `is_error` | `is_error v` | Error이면 1, 아니면 0 |
| `ok` | `ok v 기본값` | v가 Error이면 기본값 반환, 아니면 v 반환 |

```
result = num "bad"
is_error result       # → 1
ok result 0           # → 0
ok (num "42") 0       # → 42
```

### 수학 — 기본

| 함수 | 시그니처 | 설명 |
|------|----------|------|
| `floor` | `floor x` | 내림 |
| `ceil` | `ceil x` | 올림 |
| `round` | `round x` | 반올림 |
| `abs` | `abs x` | 절댓값 |
| `min` | `min a b` | 두 수 중 최솟값 |
| `max` | `max a b` | 두 수 중 최댓값 |
| `sign` | `sign x` | 부호: -1, 0, 1 |
| `clamp` | `clamp x lo hi` | x를 [lo, hi] 범위로 제한 |

### 수학 — 지수/로그

| 함수 | 시그니처 | 설명 |
|------|----------|------|
| `sqrt` | `sqrt x` | 제곱근 |
| `cbrt` | `cbrt x` | 세제곱근 |
| `pow` | `pow x y` | x의 y제곱 (`**` 연산자와 동일) |
| `exp` | `exp x` | e^x |
| `log` | `log x` | 자연로그 ln(x) |
| `log` | `log x base` | 밑이 base인 로그 |
| `log2` | `log2 x` | 밑 2 로그 |
| `log10` | `log10 x` | 밑 10 로그 |

### 수학 — 삼각함수

모든 함수는 라디안 단위를 사용합니다.

| 함수 | 시그니처 | 설명 |
|------|----------|------|
| `sin` | `sin x` | 사인 |
| `cos` | `cos x` | 코사인 |
| `tan` | `tan x` | 탄젠트 |
| `asin` | `asin x` | 아크사인 |
| `acos` | `acos x` | 아크코사인 |
| `atan` | `atan x` | 아크탄젠트 |
| `atan2` | `atan2 y x` | 2인자 아크탄젠트 |
| `hypot` | `hypot x y` | sqrt(x²+y²) |

### 수학 — 난수

| 함수 | 시그니처 | 설명 |
|------|----------|------|
| `random` | `random()` | [0, 1) 부동소수점 난수 |
| `rand_int` | `rand_int n` | [0, n) 정수 난수 |
| `rand_int` | `rand_int lo hi` | [lo, hi) 정수 난수 |

### 문자열

| 함수 | 시그니처 | 설명 |
|------|----------|------|
| `upper` | `upper s` | 대문자 변환 |
| `lower` | `lower s` | 소문자 변환 |
| `trim` | `trim s` | 앞뒤 공백 제거 |
| `split` | `split s sep` | 구분자로 분할 → 리스트 |
| `split` | `split s` | 공백으로 분할 |
| `join` | `join lst sep` | 리스트 → 문자열 |
| `join` | `join lst` | 구분자 없이 결합 |
| `replace` | `replace s old new` | 모든 `old`를 `new`로 교체 |
| `starts_with` | `starts_with s prefix` | 접두사 일치 → 1/0 |
| `ends_with` | `ends_with s suffix` | 접미사 일치 → 1/0 |
| `index_of` | `index_of s sub` | 부분문자열 위치 (없으면 -1) |
| `index_of` | `index_of lst val` | 리스트에서 값 위치 (없으면 -1) |
| `repeat` | `repeat s n` | 문자열을 n번 반복 |
| `repeat` | `repeat lst n` | 리스트를 n번 반복 |
| `char_at` | `char_at s i` | 인덱스 i의 문자 (음수 인덱스 가능) |
| `chars` | `chars s` | 문자열 → 문자 리스트 |
| `format` | `format "{}+{}={}" a b c` | `{}`를 인자로 순서대로 치환 |
| `contains` | `contains s sub` | 부분문자열 포함 여부 → 1/0 |
| `len` | `len s` | 문자 수 |

```
format "{} 더하기 {} = {}" 1 2 3    # → "1 더하기 2 = 3"
replace "hello world" "world" "C-DSL"   # → "hello C-DSL"
chars "abc"    # → ["a", "b", "c"]
```

### 리스트 — 기본

| 함수 | 시그니처 | 설명 |
|------|----------|------|
| `len` | `len lst` | 길이 |
| `push` | `push lst val` | 끝에 추가, 새 리스트 반환 |
| `pop` | `pop lst` | 마지막 제거, 새 리스트 반환 |
| `first` | `first lst` | 첫 번째 요소 (없으면 nil) |
| `last` | `last lst` | 마지막 요소 (없으면 nil) |
| `slice` | `slice lst start` | start부터 끝까지 |
| `slice` | `slice lst start end` | [start, end) 부분 리스트 |
| `concat` | `concat a b` | 두 리스트 이어 붙이기 |
| `flat` | `flat lst` | 1단계 평탄화 |
| `sort` | `sort lst` | 오름차순 정렬 (숫자/문자열) |
| `contains` | `contains lst val` | 포함 여부 → 1/0 |
| `range` | `range n` | `[0, 1, ..., n-1]` |
| `range` | `range start end` | `[start, ..., end-1]` |

### 리스트 — 고차 함수

| 함수 | 시그니처 | 설명 |
|------|----------|------|
| `map` | `map lst fn` | 각 요소에 fn 적용 |
| `filter` | `filter lst fn` | fn이 참인 요소만 |
| `reduce` | `reduce lst fn init` | 좌결합 축약 |
| `any` | `any lst fn` | fn이 참인 요소가 하나라도 있으면 1 |
| `all` | `all lst fn` | 모든 요소에 fn이 참이면 1 |
| `find_where` | `find_where lst fn` | fn이 참인 첫 요소 (없으면 nil) |
| `count` | `count lst fn` | fn이 참인 요소 개수 |
| `flat_map` | `flat_map lst fn` | map 후 1단계 평탄화 |

### 리스트 — 변환/집계

| 함수 | 시그니처 | 설명 |
|------|----------|------|
| `reverse` | `reverse lst` | 역순 리스트 |
| `unique` | `unique lst` | 중복 제거 (순서 유지) |
| `sum` | `sum lst` | 숫자 리스트 합계 |
| `product` | `product lst` | 숫자 리스트 곱 |
| `take` | `take lst n` | 앞 n개 |
| `skip` | `skip lst n` | 앞 n개 건너뛰기 |
| `zip` | `zip a b` | 두 리스트를 쌍으로 묶기 → `[[a0,b0],[a1,b1],...]` |
| `zip_with` | `zip_with a b fn` | 두 리스트를 fn으로 결합 |
| `enumerate` | `enumerate lst` | 인덱스와 쌍으로 → `[[0,v0],[1,v1],...]` |
| `group_by` | `group_by lst fn` | fn 결과 기준으로 딕트로 묶기 |

```
zip [1,2,3] ["a","b","c"]        # → [[1,"a"],[2,"b"],[3,"c"]]
zip_with [1,2,3] [4,5,6] fn a,b => a+b   # → [5, 7, 9]
enumerate ["x","y","z"]          # → [[0,"x"],[1,"y"],[2,"z"]]
group_by [1,2,3,4] fn x => ?x % 2 == 0 : "짝" : "홀"
# → {홀:[1,3], 짝:[2,4]}
```

### 딕트

| 함수 | 시그니처 | 설명 |
|------|----------|------|
| `keys` | `keys d` | 키 리스트 |
| `values` | `values d` | 값 리스트 |
| `set` | `set d key val` | 키 설정, 새 딕트 반환 |
| `get` | `get d key default` | 키 조회, 없으면 default 반환 |
| `del` | `del d key` | 키 제거, 새 딕트 반환 |
| `merge` | `merge d1 d2` | 두 딕트 합병 (d2가 d1 덮어씀) |
| `has` | `has d key` | 키 존재 여부 → 1/0 |
| `contains` | `contains d key` | 키 존재 여부 → 1/0 (`has`와 동일) |
| `len` | `len d` | 항목 수 |

```
d = {a: 1, b: 2}
get d "c" 99           # → 99 (없는 키)
has d "a"              # → 1
del d "a"              # → {b: 2}
merge {a:1} {b:2, a:9} # → {a:9, b:2}
```

### 파일 I/O

| 함수 | 시그니처 | 설명 |
|------|----------|------|
| `read_file` | `read_file path` | 파일 전체 읽기 → 문자열 |
| `write_file` | `write_file path content` | 파일 쓰기 (덮어씀), nil 반환 |
| `append_file` | `append_file path content` | 파일에 내용 추가, nil 반환 |

### HTTP / JSON

| 함수 | 시그니처 | 설명 |
|------|----------|------|
| `http_get` | `http_get url` | GET 요청 → 문자열 (실패 시 Error) |
| `http_post` | `http_post url body` | POST 요청 → 문자열 |
| `json_parse` | `json_parse s` | JSON 문자열 → 값 |
| `json_str` | `json_str v` | 값 → JSON 문자열 |

### 프로세스

| 함수 | 시그니처 | 설명 |
|------|----------|------|
| `sleep` | `sleep ms` | 밀리초 대기, nil 반환 |
| `exit` | `exit n` | 종료 코드 n으로 프로세스 종료 |

---

## 에러 전파 규칙

- Error 값은 연산, 조건, 파이프에서 자동 전파됩니다.
- 변수 대입(`x = risky()`)은 전파를 막고 Error를 저장합니다.
- `is_error`, `ok`는 전파 없이 Error를 인자로 받습니다.
- `try/catch`는 블록 내 Error를 잡아 변수에 바인딩합니다.
- `break`, `continue`, `return`은 루프/함수 경계에서만 소비됩니다.

---

## 예시

```
# 팩토리얼
fn fact n => ?n > 0 : n * fact(n-1) : 1
fact 10    # → 3628800
```

```
# 조기 반환 — 함수 본문은 단일 표현식이므로 find_where 활용
fn first_positive lst => find_where lst fn x => x > 0
first_positive [-3, -1, 4, 7]    # → 4
```

```
# break / continue
result = []
i = 0
while i < 10
  i = i + 1
  ?i % 2 == 0 : continue : nil   # 짝수 건너뜀
  ?i > 7 : break : nil           # 7 초과 종료
  result = push result i
end
print result    # [1, 3, 5, 7]
```

```
# 리스트 처리 파이프라인
[1, 2, 3, 4, 5]
  | filter fn x => x % 2 == 0
  | map fn x => x * 10
  | sum
# → 60
```

```
# 수학
sqrt 2             # → 1.4142...
log 100 10         # → 2
sin pi             # ≈ 0
atan2 1 1          # → pi/4
clamp(-5, 0, 100)  # → 0
rand_int 1 7       # 1~6 주사위
```

```
# 문자열 처리
s = "Hello, World!"
upper s                          # → "HELLO, WORLD!"
replace s "World" "C-DSL"        # → "Hello, C-DSL!"
starts_with s "Hello"            # → 1
index_of s "World"               # → 7
format "{}개 중 {}번째" 10 3     # → "10개 중 3번째"
chars "abc" | map fn c => upper c | join ""  # → "ABC"
```

```
# 딕트 처리
users = [
  {name: "Alice", age: 30},
  {name: "Bob", age: 25}
]
adults = filter users fn u => u.age >= 18
names = map adults fn u => u.name
print (join names ", ")    # → "Alice, Bob"
```

```
# 에러 처리
result = num "bad_input"
?is_error result : print "변환 실패" : print result

safe = ok (num "bad") 0     # → 0

try
  data = json_parse read_file "config.json"
  print data.host
catch e
  print "설정 로드 실패: " + e.message
end
```

```
# AI 호출 (캐시 포함)
reply = model "claude-sonnet-4-6" "피보나치 수열이란?"
print reply

# 캐시 무시 재호출
fresh = model "claude-sonnet-4-6" "오늘 날씨?" "" "true"
```

```
# HTTP + JSON
raw = http_get "https://api.example.com/data"
?is_error raw : print "요청 실패" : nil
data = json_parse raw
print data["name"]
```

```
# zip_with / enumerate 활용
a = [1, 2, 3]
b = [10, 20, 30]
zip_with a b fn x,y => x * y    # → [10, 40, 90]

pairs = enumerate ["apple","banana","cherry"]
each pairs : fn pair => print (str pair[0]) + ": " + pair[1]
```
