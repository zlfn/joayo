import "@typespec/http";
using TypeSpec.Http;

namespace Joayo;

enum SessionError {
	SessionInvalid,
}

/**
`session_id` 쿠키가 없거나 세션이 만료되었습니다.
*/
model SessionInvalidError {
	@statusCode statusCode: 401,
	@body body: ServerErrorResult<SessionError.SessionInvalid>;
}
