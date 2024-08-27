import "@typespec/http";
using TypeSpec.Http;

namespace Joayo;

@doc("유저 생성 요청데이터입니다.")
model CreateUserRequest {
	@doc("유효한 이메일 주소입니다.")
	email: string;

	@doc("비밀번호입니다. 유니코드를 지원하며, UTF-8 기준 8~128 바이트여야 합니다.")
	@minLength(8)
	@maxLength(128)
	password: string;
}

@doc("유저 생성 오류코드입니다.")
enum CreateUserError {
	EmailExist: "EmailExist",
	BadPassword: "BadPassword",
	...ServerError,
}

/**
성공적으로 유저를 생성했습니다.
*/
model CreateUserResponse {
	@statusCode statusCode: 201,
	@body body: ServerOkResult<null>,
}

/**
이메일이 서버에 이미 존재합니다.
*/
@error
model EmailExistError {
	@statusCode statusCode: 409,
	@body body: ServerErrorResult<CreateUserError.EmailExist>;
}

/**
비밀번호가 8~128 바이트가 아닙니다.
*/
@error
model BadPasswordError {
	@statusCode statusCode: 422,
	@body body: ServerErrorResult<CreateUserError.BadPassword>;
};


/**
이메일과 비밀번호로 새로운 유저를 생성합니다.  
추후 이메일 인증 전까지 계정을 활성화하지 않을 예정이 있습니다.
*/
@route("/register")
@summary("유저 생성")
@tag("user")
@post
op createUser(@body payload: CreateUserRequest): CreateUserResponse
| EmailExistError | BadPasswordError | InternalServerError;
