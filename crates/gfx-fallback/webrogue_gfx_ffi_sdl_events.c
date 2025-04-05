#include "webrogue_gfx_ffi_sdl_events.h"
#if WEBROGUE_GFX_SDL_VERSION == 2
#include "SDL.h"
#define SDL_V(v2, v3) v2
#elif WEBROGUE_GFX_SDL_VERSION == 3
#include <SDL3/SDL.h>
#define SDL_V(v2, v3) v3
#else
#error Unknown WEBROGUE_GFX_SDL_VERSION value
#endif

void webrogue_gfx_ffi_sdl_poll(webrogue_event_out_buf *event_buf,
                               void **out_buf, uint32_t *out_len) {
  event_buf->used_size = 0;

  SDL_Event event = {0};
  while (SDL_PollEvent(&event) != 0) {
    switch (event.type) {
    case SDL_V(SDL_MOUSEBUTTONDOWN, SDL_EVENT_MOUSE_BUTTON_DOWN):
      webrogue_event_encode_mouse_down(event_buf, event.button.x,
                                       event.button.y, event.button.button);
      break;
    case SDL_V(SDL_MOUSEBUTTONUP, SDL_EVENT_MOUSE_BUTTON_UP):
      webrogue_event_encode_mouse_up(event_buf, event.button.x, event.button.y,
                                     event.button.button);
      break;
    case SDL_V(SDL_MOUSEMOTION, SDL_EVENT_MOUSE_MOTION):
      webrogue_event_encode_mouse_motion(event_buf, event.button.x,
                                         event.button.y);
      break;
    case SDL_V(SDL_QUIT, SDL_EVENT_QUIT):
      webrogue_event_encode_quit(event_buf);
      break;
    }
  }
  *out_buf = event_buf->buf;
  *out_len = event_buf->used_size;
}
