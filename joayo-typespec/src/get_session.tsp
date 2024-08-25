import "@typespec/http";
using TypeSpec.Http;

namespace Joayo;

model GetSessionRequest {
	@doc("세션을 발급할 유저의 이메일 주소입니다.")
	email: string;

	@doc("세션을 발급할 유저의 비밀번호입니다.")
	password: string;
}

@doc("세션 발급 오류코드입니다.")
enum GetSessionError {
	WrongIdentity: "WrongIdentity",
	...ServerError,
}

/**
성공적으로 세션을 발급했습니다.
*/
model GetSessionResponse {
	@statusCode statusCode: 200,
	@body body: ServerOkResult<null>,
	@header "Set-Cookie": "session_id={your_session_id}",
}

/**
이메일이나 비밀번호가 틀렸습니다.
*/
@error
model WrongIdentityError {
	@statusCode statusCode: 403,
	@body body: ServerErrorResult<GetSessionError.WrongIdentity>
}

/**
유저의 이메일과 비밀번호를 통해 세션을 발급합니다.  
세션은 `set-cookie` 헤더를 통해 전달되며, 유휴 시간은 30분입니다.

세션은 웹 프론트엔드 로그인을 위해 사용되며, 언제든지 사양이 변경될 가능성이 있습니다.
외부 연동 서비스의 경우에는 토큰을 이용하세요. (구현 예정)
*/
@route("/login")
@summary("세션 발급")
@tag("user")
@post
op get_session(@body payload: GetSessionRequest): GetSessionResponse
| WrongIdentityError | InternalServerError;
