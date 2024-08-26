# JOAYO

*Like! for Everywhere* | *JOAYO!를 세상 어디에든*

웹 상의 어디에서든 이미지에 좋아요!를 누르고 한 곳에 모아 볼 수 있게 하는 토이 프로젝트  
프론트엔드 + 백엔드 + 브라우저 익스텐션으로 구성

### Big Dream (Dif from pinterest)
* 강력한 확장 프로그램과 웹 프론트엔드 연계
  - 이미지의 레퍼런스로부터 원본 이미지 로딩
  - 고정 이미지 url이 없는 이미지나 로그인이 필요한 이미지로부터 JOAYO 생성
  - 사이트 내부의 액션으로 자동 JOAYO 생성
  - 정규표현식을 이용한 커스텀 사이트 지원
* 셀프 호스트 가능 (Pinry와 유사)
* 최대한 다양한 플랫폼의 공식 지원
  - Chrome, Firefox, Safari (via WXT)
  - 애플 단축키
  - 안드로이드 애플리케이션


### How to run
```nushell
# nushell

# Dev
cd joayo-frontend; pnpm run dev
cd joayo-backend; cargo run

# Build
bazel run //joayo-frontend:build
bazel build //joayo-backend

# Test
bazel test //joayo-backend/api:test

# Run
bazel run //joayo-frontend:preview
bazel run //joayo-backend
```


## Tech Stack
![joayo drawio(3)](https://github.com/user-attachments/assets/ba71d0a7-5cfe-434d-97ca-eec139d8d9d5)


