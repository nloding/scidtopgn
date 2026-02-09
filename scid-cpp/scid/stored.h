#ifndef SCID_STORED_H
#define SCID_STORED_H

#include "game.h"

const uint MAX_STORED_LINES = 256;

class StoredLine {
  public:
    static uint Count(void) { return 0; }
    static const char * GetText(uint code) { return ""; }
    static Game * GetGame(uint code) { return NULL; }
    static bool isInitialized(void) { return false; }
};

#endif
