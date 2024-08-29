import "@typespec/http";
using TypeSpec.Http;

namespace Joayo;

@doc("비밀번호 변경 요청데이터입니다.")
model ChangePasswordRequest {
	@doc("기존 비밀번호입니다.")
	old_password: string;

	@doc("새 비밀번호입니다. 유니코드를 지원하며, UTF-8 기준 8~128 바이트여야 합니다.")
	@minLength(8)
	@maxLength(128)
	new_password: string;
}

@doc("비밀번호 변경 오류코드입니다.")
enum ChangePasswordError {
	BadNewPassword,
	WrongOldPassword,
	...SessionError,
	...ServerError,
}

/**
비밀번호 변경에 성공했습니다.
*/
model ChangePasswordResponse {
	@statusCode statusCode: 200,
	@body body: ServerOkResult<null>,
	@header "Set-Cookie": "session_id={your_session_id}",
}

/**
새 비밀번호가 8~128 바이트가 아닙니다.
*/
model BadNewPasswordError {
	@statusCode statusCode: 422,
	@body body: ServerErrorResult<ChangePasswordError.BadNewPassword>
}

/**
기존 비밀번호가 틀렸습니다.
*/
model WrongOldPasswordError {
	@statusCode statusCode: 403,
	@body body: ServerErrorResult<ChangePasswordError.WrongOldPassword>,
}

/**
세션에 대응하는 유저의 비밀번호를 변경합니다.

유저의 모든 세션을 삭제하고, 새 세션을 발급하여 `set-cookie` 헤더를 통해 전달합니다.
*/
@put
@route("/password")
@tag("user")
@summary("비밀번호 변경")
@useAuth(ApiKeyAuth<ApiKeyLocation.cookie, "session_id">)
op change_password(@body payload: ChangePasswordRequest): ChangePasswordResponse
| BadNewPasswordError | WrongOldPasswordError | SessionInvalidError | InternalServerError;
