#ifdef WEBROGUE_PDCURSES
#include "../../../external/pdcurses/curses.h"
#endif

#ifdef WEBROGUE_NCURSES
#include <ncurses.h>
#define nc_getmouse(x) getmouse(x)
#endif

#if !defined(WEBROGUE_NCURSES) && !defined(WEBROGUE_PDCURSES)
#error Define WEBROGUE_NCURSES or WEBROGUE_PDCURSES to use curses
#endif
