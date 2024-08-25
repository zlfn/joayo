<!-- 문서 오버뷰의 마크다운입니다. 실제로 이 파일을 사용하지는 않으므로 직접 복사해야 합니다.-->
## JOAYO API DOCS
JOAYO의 API 문서입니다.

### 기본적인 응답형태
모든 응답은 기본적으로 아래와 같은 형태로 주어집니다.
```js
{
    "type": "Ok" | "Error",
    "data": Object | null | string
}
```

* `type`: `"Ok"`나 `"Error"`로 요청의 성공 여부를 나타냅니다.  
* `data`: 요청이 성공했을 경우 요청한 데이터의 객체 혹은 응답 데이터가 없는 요청의 경우 `null`이 주어집니다.  
          요청이 실패했을 경우 `string`으로 상세 에러코드가 주어집니다.

### JSON 에러
서버가 JSON의 파싱에 실패했을 경우, 아래와 같은 응답이 주어집니다.
```js
{
    "type": "Error",
    "data": JsonErrorType as string
}
```

`JsonErrorType`으로 주어질 수 있는 값은 아래와 같습니다.
* `"JsonDataError"`: JSON의 문법 혹은 JSON의 형태가 필요료 하는 형식과 맞지 않을 경우
* `"JsonContentTypeError"`: HTTP 헤더의 `Content-Type`이 `application/json`이 아닌 경우
* `"JsonBytesError"`: 서버가 JSON 파싱 중에 Rust의 `Bytes` 와 관련된 오류를 일으킨 경우

아래는 주어지지 않으나, 추후 주어질 가능성이 있는 값입니다.
* `"JsonSyntaxError"`: 원래는 JSON의 문법이 틀렸을 때 `JsonDataError`가 아닌 이 오류가 반환되어야 하나, 백엔드가 사용하는 프레임워크인 Axum과 관련된 문제로 실제로 사용되지 않음.
* `"JsonUnknownError"`: Axum의 JSON 파싱 로직이 바뀌었을 때를 대비한 오류
