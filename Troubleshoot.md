# Troubleshoot Guide

이 문서는 개발 중 발생한 문제와 해결책을 기록하여 동일한 실수를 방지합니다.

---

## CWD 오버레이로 인한 터미널 마지막 행 표시 문제

### 증상
- 여러 pane으로 분할된 상태에서 활성 pane의 터미널 마지막 행이 pane 영역을 벗어나 보이지 않음
- CWD(Current Working Directory) 오버레이가 각 pane의 첫 번째 행에 표시될 때 발생

### 근본 원인
1. **PTY 크기와 표시 영역 불일치**: CWD 오버레이가 pane의 첫 번째 행을 차지하지만, PTY(터미널)에 보고되는 크기는 전체 pane 높이를 사용
2. **렌더링 오프셋 문제**: 터미널 내용이 `cwd_row_offset`만큼 아래로 밀려 렌더링되지만, 터미널은 여전히 전체 높이만큼 내용을 출력
3. **결과**: 터미널이 n행을 출력하지만 화면에는 n-1행만 표시 가능하여 마지막 행이 잘림

### 잘못된 접근 (하지 말 것)
- `stable_range` 시작점을 `cwd_row_offset`만큼 이동시키는 방법
  - 이렇게 하면 마지막 행은 보이지만 **첫 번째 행이 누락됨**
  - 터미널 크기 자체가 문제이므로 렌더링 범위 조정은 해결책이 아님

### 올바른 해결책
**PTY에 보고되는 높이를 CWD 오버레이가 있을 때 1행 줄임**

#### 수정 위치 1: `wezterm-gui/src/termwindow/resize.rs`
```rust
// apply_dimensions() 함수 내, tab.resize(size) 후:
let panes = tab.iter_panes_ignoring_zoom();
if panes.len() > 1 {
    let cell_height = self.render_metrics.cell_size.height as usize;
    for pos in panes {
        let adjusted_size = TerminalSize {
            rows: pos.height.saturating_sub(1),
            cols: pos.width,
            pixel_height: pos.pixel_height.saturating_sub(cell_height),
            pixel_width: pos.pixel_width,
            dpi: size.dpi,
        };
        let _ = pos.pane.resize(adjusted_size);
    }
}
```

#### 수정 위치 2: `wezterm-gui/src/termwindow/render/paint.rs`
```rust
// paint_pass() 함수 내, get_panes_to_render() 후:
if panes.len() > 1 {
    let cell_height = self.render_metrics.cell_size.height as usize;
    for pos in &panes {
        let dims = pos.pane.get_dimensions();
        // PTY 높이가 pane 높이와 같을 때만 조정 (중복 조정 방지)
        if dims.viewport_rows == pos.height {
            let adjusted_size = wezterm_term::TerminalSize {
                rows: pos.height.saturating_sub(1),
                cols: pos.width,
                pixel_height: pos.pixel_height.saturating_sub(cell_height),
                pixel_width: pos.pixel_width,
                dpi: dims.dpi,
            };
            let _ = pos.pane.resize(adjusted_size);
        }
    }
}
```

### 왜 두 곳에서 수정해야 하는가?
1. **resize.rs**: 창 크기 변경 시 호출됨
2. **paint.rs**: split 발생 시에도 PTY 크기가 올바르게 조정되도록 함 (split은 `apply_dimensions()`를 호출하지 않을 수 있음)

### 핵심 교훈
- GUI 오버레이가 터미널 표시 영역을 줄이면, **PTY에 보고되는 크기도 함께 조정**해야 함
- 렌더링 범위만 조정하면 내용 누락 문제가 발생함
- vim, less 등 터미널 프로그램이 정확한 화면 크기를 알아야 올바르게 렌더링할 수 있음

### 관련 파일
- `wezterm-gui/src/termwindow/render/pane.rs`: 터미널 내용 렌더링, `cwd_row_offset` 계산
- `wezterm-gui/src/termwindow/render/split.rs`: CWD 오버레이 렌더링 (`paint_pane_cwd_overlays`)
- `wezterm-gui/src/termwindow/resize.rs`: 창 크기 조정
- `wezterm-gui/src/termwindow/render/paint.rs`: 메인 렌더링 루프
- `mux/src/tab.rs`: pane 크기 관리, `PositionedPane` 구조체
