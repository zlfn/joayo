import "@typespec/http";
using TypeSpec.Http;

namespace Joayo;

@doc("세션 삭제 오류코드입니다.")
enum DeleteSessionError {
	...SessionError,
	...ServerError,
}

/**
세션을 삭제하는데 성공했습니다.
*/
model DeleteSessionResponse {
	@statusCode statusCode: 200,
	@body body: ServerOkResult<null>,
}


/**
로그인되어 있는 세션을 삭제합니다.
*/
@delete
@route("/session")
@summary("세션 삭제")
@tag("user")
@useAuth(ApiKeyAuth<ApiKeyLocation.cookie, "session_id">)
op delete_session(): DeleteSessionResponse
| SessionInvalidError | InternalServerError;
