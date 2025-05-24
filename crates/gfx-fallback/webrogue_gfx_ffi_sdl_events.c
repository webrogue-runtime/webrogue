#include "webrogue_gfx_ffi_sdl_events.h"
#include "webrogue_event_encoder.h"
#include <SDL3/SDL.h>

void webrogue_gfx_ffi_sdl_poll(webrogue_event_out_buf *event_buf,
                               void **out_buf, uint32_t *out_len) {
  event_buf->used_size = 0;

  SDL_Event event = {0};
  while (SDL_PollEvent(&event) != 0) {
    switch (event.type) {
    case SDL_EVENT_MOUSE_BUTTON_DOWN:
    case SDL_EVENT_MOUSE_BUTTON_UP:
      webrogue_event_encode_mouse_button(event_buf, event.button.button,
                                         event.type ==
                                             SDL_EVENT_MOUSE_BUTTON_DOWN,
                                         event.button.x, event.button.y);
      break;
    case SDL_EVENT_MOUSE_MOTION:
      webrogue_event_encode_mouse_motion(event_buf, event.button.x,
                                         event.button.y);
      break;
    case SDL_EVENT_QUIT:
      webrogue_event_encode_quit(event_buf);
      break;
    case SDL_EVENT_KEY_DOWN:
    case SDL_EVENT_KEY_UP:
      webrogue_event_encode_key(event_buf, event.type == SDL_EVENT_KEY_DOWN,
                                event.key.scancode);
      break;
    case SDL_EVENT_WINDOW_RESIZED:
      webrogue_event_encode_window_resized(event_buf);
      break;
    case SDL_EVENT_TEXT_INPUT: {
      const char *c = event.text.text;
      do {
        webrogue_event_encode_text_input(event_buf, *c);
      } while (*(c++)); // encodes \0 as well
    } break;
    }
  }
  *out_buf = event_buf->buf;
  *out_len = event_buf->used_size;
}
