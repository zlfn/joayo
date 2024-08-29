import "@typespec/http";
using TypeSpec.Http;

namespace Joayo;

enum ServerError {
	InternalServerError,
}

/**
내부 서버 오류입니다.
*/
@error
model InternalServerError {
	@statusCode statusCode: 500,
	@body body: ServerErrorResult<ServerError.InternalServerError>;
}
