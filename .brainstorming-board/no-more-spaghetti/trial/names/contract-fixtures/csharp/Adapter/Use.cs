using Acme.Core;
using Acme.Stray;
using Acme.Gen;
#if FAST
using Acme.Core.Fast;
#else
using Acme.Core.Slow;
#endif
namespace Acme.Adapter;
public class Use { void f(string n) { System.Type.GetType(n); } }
