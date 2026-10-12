#include <stdint.h>
#include <stdlib.h>
#include <libwr4c/wr4c.h>

int main(void) {
  wr4c_window_t window;
  wr4c_make_window(&window);

  int width = 0;
  int height = 0;
  wr4c_gl_size(window, &width, &height);
  if (width < 2 || height < 2)
    return 1;

  uint32_t *pixels = malloc((size_t)width * height * sizeof(*pixels));
  if (pixels == NULL)
    return 2;

  for (int y = 0; y < height; ++y) {
    for (int x = 0; x < width; ++x) {
      uint32_t red = (uint32_t)(x * 13 + y * 3) & 0xff;
      uint32_t green = (uint32_t)(x * 5 + y * 19) & 0xff;
      uint32_t blue = (uint32_t)((x ^ (y * 3)) * 11) & 0xff;
      if (((x / 4) + (y / 4)) & 1)
        red = 0xff - red;
      pixels[(size_t)y * width + x] = (red << 16) | (green << 8) | blue;
    }
  }

  wr4c_present_pixels(window, (uint8_t *)pixels,
                      (uint32_t)((size_t)width * height * sizeof(*pixels)));
  free(pixels);
  return 0;
}
