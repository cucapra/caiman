2 Oct 2021
Want several levels that build on each other:
1. wgpu - a direct encoding of wgpu
2. factorized wgpu - wgpu split into both a resource-free spec and resource-managing schedule?
3. easy wgpu - using the hardware functionality provided by wgpu, can we build a API more amenable to static analysis and optimization?

Goals:
1. Build an api that's easier to use and is reasonably performant
    a. Make it suitable for students in intro graphics courses that are learning graphics and don't want to deal with the boilerplate and complexity that these APIs have for making things fast.
2. Demonstrate that we can safely and conveniently expose functionality that wgpu restricts, leading to more performant applications.  Critically, many restrictions are made in wgpu due to the need for runtime checks.

Phases?
1. CPU code in rust, GPU code in glsl/spirv, glue together using new system
2. Expand capability of the glue later into CPU (webassembly) and GPU code (spirv)
3. Unified language (webassembly?)

7 Oct 2021
For the immediate future, I think starting with two separate languages and introducing a "GPU Coordinator" language block that glues them together is the right model.

The GPU Coordinator block captures that there's some code we definitely want on the CPU, there's some code that we definitely want on the GPU, and then there's a fuzzy interface that we want to quickly and easily manipulate based on differences in underlying cost models.

This also keeps scope to the interface rather than the whole unified language problem of CUDA and SYCL, and presents a more gradual approach towards a potential unification.  The interface is where many of the interesting problems are, anyway.

Ultimately, I think I want a language model for explaining limitations in the APIs and why they're so painful.

15 Oct 2021

I created this "definition" and "ir" separation because I needed a user friendly way to edit programs, but I'm increasingly feeling like the definition part is going to be throwaway work in the near future.  I'm going to almost certainly want a better definition language than what I can expose via TOML.

19 Oct 2021

Yeah, it doesn't make much sense to put that much effort into the definition part.  I think it's good as a functioning sketch of what a slightly less painful UX for the system would look like, but the end goal really would be to ditch it as soon as possible in favor of compilation from a different system.  At this point, I'm putting more effort into the IR so I can get a better feel for the transforms in action.

20 Oct 2021

Stages, roughly, are:
1. high level function soup with no notion of where things are computed
2. Map values into version of language with scope functors around everything to express CPU and GPU computation
