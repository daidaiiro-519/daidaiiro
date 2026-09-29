#include "app/core/name.h"
#include "app/stray/x.h"
#include "app/gen/made.h"
#include CONFIG_HEADER
#ifdef FAST
#include "app/core/fast.h"
#else
#include "app/core/slow.h"
#endif
void* h = dlopen(path, 0);
