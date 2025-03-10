#include "SDL.h"
#include "webrogue_gfx_ffi_sdl2_events.h"

void webrogue_gfx_ffi_sdl2_poll(webrogue_event_out_buf* event_buf, void** out_buf, uint32_t* out_len) {
  event_buf->used_size = 0;
  
  #define RETURN *out_buf = event_buf->buf; *out_len = event_buf->used_size; return;
  SDL_Event event = { 0 };
  while (SDL_PollEvent(&event) != 0) {
    switch (event.type) {
      case SDL_MOUSEBUTTONDOWN: {
        webrogue_event_encode_mouse_down(event_buf, event.button.x, event.button.y, event.button.button);
        RETURN
      } break;
      case SDL_MOUSEBUTTONUP: {
        webrogue_event_encode_mouse_up(event_buf, event.button.x, event.button.y, event.button.button);
        RETURN
      } break;
      case SDL_MOUSEMOTION: {
        webrogue_event_encode_mouse_motion(event_buf, event.button.x, event.button.y);
        RETURN
      } break;
      case SDL_QUIT: {
        webrogue_event_encode_quit(event_buf);
        RETURN
      } break;
    }
  }
  RETURN;
  #undef RETURN
}
