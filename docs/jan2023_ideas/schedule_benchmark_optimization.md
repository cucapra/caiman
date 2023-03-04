Given some baseline benchmarks put together (see `schedule_benchmarks.md`),
which I will be doing at some level hopefully soon, and that project would
expand on, then we also need to hand-optimize those benchmarks.  What exactly
hand-optimization entails is somewhat vague, however, and in some sense what we
should focus on is "best effort".

As a result, I will make some guesses below for some obvious hand-optimization
direction assuming a very naive initial implementation, but part of the point of
such a project is to provide more extensive hand-optimized testing.

### Hand Optimization Ideas

1. Removing scheduled code, probably some variation of DCE or CSE.  Basically
   finding operations that are either duplicated or never run and removing those

2. Loop management, so removing duplicate loop code (through recursion cause
   caiman of course).  Hand loop optimizations tend to be a pain though, so
   probably not too much interesting here that isn't a "micro-optimization"

3. To start the more "interesting" stuff, moving scheduling operations to
   "cluster" them or do things earlier.  So scheduling data movement so it only
   happens once, or happens as early as possible to avoid a bottleneck, or more
   advanced so it doesn't interfere with another thing that should be done
   earlier
   
4. Selection / control flow logic to minimize data movement branches.  When
   moving data, you really don't want to spend the time to move data if it's
   just going to be overwritten, so finding ways to help with
   semi-branch-prediction could be interesting

I'll just stop, cause ultimately it will just be trying things, and that's sort
of the point.  Having another person trying things would help a lot.