#include <stdio.h>

int main(void) {
  FILE *resource = fopen("/bundle/nested/message.txt", "rb");
  if (resource == NULL) {
    fputs("failed to open bundled resource\n", stderr);
    return 1;
  }

  char buffer[64];
  size_t bytes_read;
  while ((bytes_read = fread(buffer, 1, sizeof(buffer), resource)) > 0) {
    if (fwrite(buffer, 1, bytes_read, stdout) != bytes_read) {
      fclose(resource);
      return 2;
    }
  }

  if (ferror(resource)) {
    fclose(resource);
    return 3;
  }
  if (fclose(resource) != 0)
    return 4;
  return 0;
}
