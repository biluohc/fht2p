#include <stdio.h>
#include <windows.h>

BOOL CtrlHandler(DWORD fdwCtrlType);

int main(int argc, char **argv)
{
  if (SetConsoleCtrlHandler((PHANDLER_ROUTINE)CtrlHandler, TRUE))
  {
    while (1){}
  }
  else
    return 0;
}

BOOL CtrlHandler(DWORD fdwCtrlType)
{
  switch (fdwCtrlType)
  {
  case CTRL_C_EVENT:
    printf("Ctrl-C event\n\n");
    exit(0);
    return (TRUE);

  case CTRL_CLOSE_EVENT:
    printf("Ctrl-Close event\n\n");
    return (TRUE);

  case CTRL_BREAK_EVENT:
    printf("Ctrl-Break event\n\n");
    return FALSE; // pass thru, let the system to handle the event.

  case CTRL_LOGOFF_EVENT:
    printf("Ctrl-Logoff event\n\n");
    return FALSE; // pass thru, let the system to handle the event.

  case CTRL_SHUTDOWN_EVENT:
    printf("Ctrl-Shutdown event\n\n");
    return FALSE; // pass thru, let the system to handle the event.

  default:
    return FALSE;
  }
}