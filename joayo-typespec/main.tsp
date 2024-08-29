import "@typespec/http";
import "@typespec/rest";
import "@typespec/openapi";
import "@typespec/openapi3";

import "./common/session_error.tsp";
import "./common/server_error.tsp";

import "./src/create_user.tsp";
import "./src/change_password.tsp";
import "./src/get_session.tsp";
import "./src/check_session.tsp";
import "./src/delete_session.tsp";

using TypeSpec.Http;
using TypeSpec.OpenAPI;

@service({
	title: "JOAYO API"
})
@server("https://localhost:7878", "")
@info({
	version: "0.0.0",
	description: """
	## JOAYO API DOCS
	JOAYO의 API 문서입니다.

	### 기본적인 응답형태
	모든 응답은 기본적으로 아래와 같은 형태로 주어집니다.
	```ts
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
	```ts
	{
	    "type": "Error",
	    "data": JsonErrorType as string
	}
	```

	`JsonErrorType`으로 주어질 수 있는 값은 아래와 같습니다.
	* `"JsonDataError"`: JSON의 형태가 필요로 하는 형식과 맞지 않을 경우
	* `"JsonSyntaxError"`: JSON의 문법이 틀린 경우
	* `"JsonContentTypeError"`: HTTP 헤더의 `Content-Type`이 `application/json`이 아닌 경우
	* `"JsonBytesError"`: 서버가 JSON 파싱 중에 Rust의 `Bytes` 와 관련된 오류를 일으킨 경우
	"""
	,
	contact: {
		name: "zlfn",
		email: "ung@zlfn.space"
	}
})
namespace Joayo;

union ServerResult<Data, Error> {
	ServerOkResult<Data>,
	ServerErrorResult<Error>
}

model ServerOkResult<Data> {
	@doc("`Ok`나 `Error`로 요청의 성공여부를 나타냅니다.")
	type: "Ok";
	@doc("요청한 데이터입니다.")
	data: Data;
}

model ServerErrorResult<Error> {
	@doc("`Ok`나 `Error`로 요청의 성공여부를 나타냅니다.")
	type: "Error";
	@doc("오류의 유형입니다.")
	data: Error;
}

