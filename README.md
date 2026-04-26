# RustMapleStory

Rust + Bevy 2D 횡스크롤 액션 프로토타입입니다.

## 실행

```bash
cargo run --features bevy/dynamic_linking --release
```

## 조작

- `A` / `D`: 좌우 이동
- `Space`: 점프
- `Ctrl`: 검은 공 투사체 발사

## 현재 구현된 요소

- 플레이어 이동 / 점프 / 중력
- 바닥 및 발판 AABB 충돌
- 카메라 플레이어 추적
- 몬스터 플레이어 추적 AI
- 투사체 발사 및 몬스터 충돌 제거
