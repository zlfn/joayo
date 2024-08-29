import "@typespec/http";
using TypeSpec.Http;

namespace Joayo;

@doc("세션 확인 응답데이터입니다.")
model CheckSessionData {
	@doc("세션에 대응되는 유저의 이메일입니다.")
	email: string
}

@doc("세션 확인 에러코드입니다.")
enum CheckSessionError {
	...SessionError,
	...ServerError,
}

/**
세션이 확인되었습니다.
*/
model CheckSessionResponse {
	@statusCode statusCode: 200,
	@body body: ServerOkResult<CheckSessionData>,
}


/**
세션이 유효한지 확인하고, 세션에 대응하는 유저의 이메일을 응답합니다.

다른 요청과 동일하게, 세션의 유효 시간도 연장합니다.
*/
@get
@route("/session")
@summary("세션 확인")
@tag("user")
@useAuth(ApiKeyAuth<ApiKeyLocation.cookie, "session_id">)
op checkSession(): CheckSessionResponse
| SessionInvalidError | InternalServerError;
