#include "windows.h"

void webrogue_aot_main();

#if defined(WR_WIN_TYPE_gui)
int WINAPI WinMain(HINSTANCE hInstance, HINSTANCE hPrevInstance,
                   LPSTR lpCmdLine, int nCmdShow)
#elif defined(WR_WIN_TYPE_console)
int main(int argc, char *argv[])
#else
#error unknown WR_WIN_TYPE_* value
#endif
{
  webrogue_aot_main();
  return 0;
}
