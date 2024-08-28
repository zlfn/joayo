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
	Unauthorized: "Unauthorized",
	SessionInvalid: "SessionInvalid",
	...ServerError,
}

/**
세션이 확인되었습니다.
*/
model CheckSessionResponse {
	@statusCode statusCode: 201,
	@body body: ServerOkResult<CheckSessionData>,
}

/**
`session_id` 쿠키가 없습니다.
*/
model UnauthorizedError {
	@statusCode statusCode: 401,
	@body body: ServerErrorResult<CheckSessionError.Unauthorized>
}

/**
세션이 만료되었거나 `session_id`가 유효하지 않습니다.
*/
model SessionInvalidError {
	@statusCode statusCode: 403,
	@body body: ServerErrorResult<CheckSessionError.SessionInvalid>
}

/**
세션이 유효한지 확인합니다.
다른 요청과 동일하게, 세션의 유효 시간도 연장합니다.
*/
@route("/login")
@summary("세션 확인")
@tag("user")
@useAuth(ApiKeyAuth<ApiKeyLocation.cookie, "session_id">)
@get
op checkSession(): CheckSessionResponse
| UnauthorizedError | SessionInvalidError | InternalServerError;
