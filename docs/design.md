# Caiman IR Design

## Motivation

Caiman is a programming language IR and compiler for rapidly exploring the design space of programs using both a CPU and GPU.  Portably programming for mixed GPU/CPU programs generally entails the use of DirectX, Vulkan, or Metal.  These API designs feature a common point of friction: CPU functions cannot directly invoke GPU functions.  Instead, the user must, manually, load a portable bytecode encoding the GPU function, construct a pipeline object that encodes all state necessary to complete the compilation, create buffers for staging data transfers between CPU and GPU, create binding objects (descriptors) to specify resources to be used by a pipeline, and create an encoding object to bind all elements together for submission to a queue.  These APIs also tend to expect manual programmer management and tracking of concurrent access hazards and lifetimes for resources in use by the CPU and GPU.  Metal and WebGPU provide stronger safety guarantees, but do so by heavily restricting the API's capability and inferring the optimal resource management at runtime using heuristics that the user has no control over.

In a GPGPU compute context, languages like CUDA or SYCL provide a single-source unified programming language that allows convenient code movement across the CPU and GPU boundary.  In these systems, functions on both the CPU and GPU can be analyzed by the same compiler to infer most of the boilerplate in a function call from CPU to GPU and code existing within a common subset of the language can migrate unchanged across the boundary.  This facilitates rapid development and exploration, but has a critical limitation: the similarities between the two sides of the boundary are syntactic but not semantic.  That is, a GPU function must access memory that is marked as accessible from the GPU and a CPU function must access memory that is marked as accessible from the CPU.  This, in general, reflects an underlying physical reality of the memory subsystem.  For data to be efficiently accessible on one, it must be physically copied a significant distance from the other.  This means the programmer must be aware of resource placement at all times.  In DirectX, Vulkan or Metal, the programmer must manually copy data between buffers.  CUDA's unified virtual memory and SYCL's unified sparse memory provide a convenient mechanism for passing pointers between CPU and GPU with minimal code changes, but at the cost of unpredictable latency as the underlying memory subsystem fetches the data.  This capability is non-portable and is generally undesirable for a low-latency environment.

The IR of Caiman is an attempt at a middle ground between the two approaches that borrows the philosophy of separation of functional and scheduling concerns from the Halide image processing language.  By factoring out resource management and movement as a separate (scheduling) language, the IR facilitates exploration of the design space of CPU/GPU interaction without hiding the underlying costs.

## Base Design

Most real-time rendering applications are designed around a render loop in which the visible scene must be updated and rerendered within the span of 7, 16, or sometimes 33 miliseconds (144 Hz, 60 Hz, or 30 Hz respectively).  Even for a slightly less-than-naïve implementation, this requires careful management of resource synchronization to avoid pipeline bubbles between the CPU and GPU.  For example, it is standard practice to begin issuing work to the GPU for the next frame while the prior frame's GPU work is still in the queue.  This entails management of several frames of resources and synchronization on points in time well before the most recently submitted work.  More sophisticated renderers must coordinate multiple render passes with resource uploads to the GPU, which may depend on reading information from the GPU to the CPU to know which resources must be uploaded.

Caiman's focus on the interface between the CPU and GPU means that it is mostly oriented around facilitating latency related optimizations and tight resource management at what is traditionally the graphics API layer executing on the CPU.  As such, as a near term goal, it is less concerned with decomposing or transforming shaders or compute kernels within a bulk GPU dispatch, unlike languages such as Halide, but is theoretically composable with such a system.  The intent is that one writes their shader code in a standard shading language (WGSL or any language that compiles to SPIRV) and uses Caiman to connect it with a host application in place of direct graphics API calls on the CPU.  It is also designed to allow progressive embedding in a hosting Rust and WebGPU application.  A programmer may reimplement only parts of their application in Caiman and interface it with their existing Rust and WebGPU code with minimal overhead.

In Caiman's IR, programs are split into two parts: a functional part, which specifies the transformation of values in a program, and a scheduling part, which specifies when and where the values are computed and stored.  The functional (value) part is, abstractly, a directed acyclic dataflow graph of pure functions (with multiple returns for some nodes like function calls).  Any logical dependency must appear as inputs to a functional node and all changes to state must appear as an output of the node.  Evaluation order is meaningless for functional nodes.  The scheduling part is, abstractly, a higher order function (a scheduling function) that takes a value function as input and produces a scheduled procedure, a linear sequence of side-effecting commands that implement functional nodes by operating on resources.  A scheduling function is therefore responsible for implementing the evaluation of functional nodes.  This separation ensures that a schedule can be completely removed or altered while trivially preserving the original functional specification.  A programmer can first define the functional behavior and then explore the design space of optimizations without fear of breaking the intended functionality.

This minimizes the false coupling of the underlying algorithms to a particular resource.  Functional nodes, when appearing to a scheduling function, may have particular requirements on the resource placement of their inputs or outputs.  For example, a compute kernel dispatch (which is necessarily scheduled on the GPU) can only receive data arguments that are resident on the GPU and can only output data that is resident on the GPU, but its dimension arguments must be resident on the CPU.  Meanwhile, a call to a CPU function (which is necessarily scheduled for execution on the CPU) can (currently) only input and output CPU resident data.  Using the output of one as the input of the other requires an explicit or implicit resource transition in the scheduling part, making visible and checkable a design trade-off of importance to performance-minded users.

At minimum, Caiman provides a guarantee that it will not accept an invalid schedule.  Scheduling functions can be statically verified to implement the provided functional nodes.  (*The following won't be true again until the explicator is reimplemented*) However given a partial specification of constraints, under most circumstances, at least one valid schedule will be practically computable in a schedule inference process called "explication".  This permits initial implementation of the functional part without implementation of the scheduling part and also reduces the boilerplate needed in the average manually written schedule.  Caiman defaults to a naïve, predictable schedule inference in cases of partial schedules so that a schedule with partial annotations is both sound and complete with respect to the language with full annotations.

Caiman is currently designed around the expectation that the functional part will be written first and then the scheduling part will be written much later when the functional part has solidified.  As such, schedules are generally expected to be brittle.  A slight change in the functional part may invalidate the schedule, requiring a rewrite of the schedule to recover performance.  A future extension may explore schedules robust to larger classes of change in the functional part.  As a further goal, similar to Halide's autoscheduler, it's hoped that most schedules will eventually be automatically generated, but human-readable.

## Caiman IR design

Caiman's IR organizes programs into pipelines (procedures and coroutines) that connect funclets (basic blocks).  The operations and nodes in a funclet are encoded as an array where each entry is a node or operation that can only depend on nodes with an index less than its own.  The tail edge (terminator node) for the funclet is encoded separately from the array and captures control flow (if control flow is supported for the relevant sublanguage).

There are a small number of structural nodes that appear in most sublanguages:
- `Phi(index)`
- `ExtractResult(tuple, index)`

"Phi" nodes represent explicit funclet arguments (they are not true phi nodes) and a funclet with `n` inputs must have exactly `n` phi nodes and they must appear as the first `n` nodes in the funclet.  Some nodes may have multiple returns.  A node (the `tuple` field) with `n` returns must have exactly `n` `ExtractResult` nodes appear as the `n` nodes following the node with multiple returns.  `ExtractResult` may not appear elswhere.  These restrictions facilitate quick lookup of the relevant structural nodes throughout the compiler.

The name "funclet" is shamelessly taken from https://github.com/WebAssembly/funclets/blob/main/proposals/funclets/Overview.md
The structure of funclets is loosely similar to cranelift (https://github.com/bytecodealliance/wasmtime/blob/main/cranelift/docs/ir.md)

## The (Value) Functional Language

The functional part of a caiman program is specified via an acyclic dataflow graph for each value funclet where no evaluation order is assumed.

A "value function" is an equivalence class of value funclets, defined by the user.  When scheduled, any value funclet may be subsituted for any other value funclet within the equivalence class.  Caiman does not attempt to prove funclet equivalence.  The intent is that the user implements equivalent implementations of leaf functions and then glues them together in caiman.  Future versions of caiman may support directly generating leaf functions from caiman code.

Besides `Phi` and `ExtractResult`, the remaining value-language nodes are:
- `constant(v, t)`
	- Represents the raw constant value `v` of type `t`
	- This is currently encoded as two nodes `ConstantInteger` and `ConstantUnsignedInteger` but should be merged
- `call_f(q_0, q_1, q_2, ...)` where `f` is a value function and `q_0, q_1, q_2, ...` are nodes
	- Represents the (pseudo-tuple-valued) result of invoking any funclet implementing the value function `f` with the arguments `q_0, q_1, q_2, ...`
	- This operator may result in unbounded recursion
- `select(q_c, q_t, q_f)` where `q_c, q_t, q_f` are nodes.
	- Represents the value of `q_t` if `q_c` is nonzero, otherwise it represents the value of `q_f`
- `callextgpu_{f,k}(q_0, q_1, q_2, ...)` where `f` is an external gpu functionm, k is the number of dimensions, and `q_0, q_1, q_2, ...` are nodes
	- An internal intrinsic operator for implementing leaf value functions via FFI
	- Represents the (pseudo-tuple-valued) result of invoking the external gpu kernel `f` with `k` (up to 3) dimensions, where the first `k` nodes of `q_0, q_1, q_2, ...` are the number of threads in each dimension, and the remaining `q_k, ...` are the arguments to the kernel itself (the exact mapping of arguments and results to bindings is defined in the FFI data structures)
- `callextcpu_f(q_0, q_1, q_2, ...)` where `f` is an external gpu function and `q_0, q_1, q_2, ...` are nodes
	- An internal intrinsic operator for implementing leaf value functions via FFI
	- Represents the (pseudo-tuple-valued) result of invoking the external cpu callback `f`, where `q_0, q_1, q_2, ...` are arguments to the function.  For now, all arguments are immutable (this will likely change) and all results must be by-value.

## Pipelines and Memory Management

A pipeline in caiman is an object that may have several stages of execution invocable from the host language (rust), where each stage leaves the pipeline in a known state suitable for the next.  A pipeline that only has one stage of execution is effectively a function, while a pipeline with multiple (potentially infinitely many) is a coroutine.  An earlier version of caiman had host-selectable resumption points making those pipelines effectively objects (in the OOP sense), but this capability has been removed (for now).

The scheduling language is designed assuming that functions cannot allocate or deallocate resources such as buffers unless the maximum capacity is statically known.  There are (currently) exceptions to this for testing/debugging reasons, but it's not expected that these will be supported for long.  This means that all pipelines are finite state machines and must be given their resources by the host code.  For multi-stage pipelines, re-entry points represent state transitions invoked explicitly by the hosting environment.  Each entry point consumes the previous pipeline object and produces a new pipeline object.  Methods may receive borrowed references to external resources as input or output arguments.  These may be accessed through special variables known as "slots" (discussed below and in the IR section).  Slots containing outputs must be allocated from a host-provided resource that must be in-scope at the time of returning to the host. This is enforced through a mechanism called a "spatial tag".

### Defining pipelines

Pipelines must be defined in the caiman IR by providing, minimally, an identifier for the host code to use and a scheduling funclet to initialize the pipeline (the entry funclet).  The output type of the entry funclet determines the output type of the whole pipeline.  For coroutines (and only coroutines), a set of yield points must be defined, which specify a set of identifiers and known program states for the host to interact with the yielded pipeline.  The entry funclet is compiled into a `start()` method on the pipeline instance, while the yield points are compiled into `resume_at_*` methods on the pipeline instance (where the `*` is the identifier provided in the definition of the yield point).  Each funclet invocation returns a FuncletResult type which can be queried to determine the state of the pipeline and readied for the next stage of the pipeline with the `prepare_next()` method.

## The Schedule Execution/Static Analysis Model

A scheduling function can only distiguish nodes by their dependencies and placement requirements, so, for example, any binary operation requiring all resources to be on the CPU will behave identically as far as the schedule is concerned.

### Queues

Much like the GPU APIs it is built on, Caiman adopts the in-order queue as an organizing principle.  A system may contain multiple threads of execution with different capabilities and each is referred to as a "place".  Each place has a single queue and all work must appear to complete in FIFO order.  Queues coordinate through fences which signal asynchronously to notify other queues of completion.  This may be extended in the future to allow more sophisticated synchronization patterns on more realistic work execution models.

Caiman programs are executed from the perspective of a (usually) single, centralized Coordinator responsible for encoding and issuing commands to other queues.  The Coordinator logically executes in the "Local" queue, which (in the current implementation) corresponds to the calling thread of execution of the host program.  As such, the Coordinator may (currently) be reasoned about in terms of performance as executing on the CPU.  However, this model will likely need to change to support indirect work dispatch on the GPU and parallel command encoding.

### Timeline Funclets

Operations that result in communication between places have an associated "timeline tag" that specifies where, in a special funclet encoding the synchronization graph (a "timeline" funclet), the current operation is occuring.  Submissions occur on a `SubmissionEvent` node and synchronizations occur on a `SynchronizationEvent` node.  Currently, all such operations must occur on the local coordinator (the "Local" place).  However, this graph is enough to encode the last known state of the gpu according to the coordinator (the round trip of local -> gpu -> local).

When an operation that advances the timeline is performed, the type and tags of slots and resources currently in scope may change.  A submission moves `Encoded` nodes to the `Submitted` state and sets the timeline tag to the current timeline, and a synchronization moves `Submitted` nodes into the `Ready` state based on which nodes have the same timeline tag as the fence used to synchronize.

### Slots

Slots are the key piece of state in schedules.  They do not hold values themselves.  Instead, they are references that associate a subexpression of an invocation of a value funclet (defined via a "value tag") to the corresponding subregion of a resource expected to hold that value at some point in the future.  At runtime, they are just pointers.

Each slot has, minimally, a type containing three pieces of information: a storage type defining which native type to use for holding the value, a place defining which queue will compute the value, and a stage marker that says what the status of the resource is known to be.  The stage state machine contains the following states:

- `Unbound`
	- There is no resource bound to this slot, but a later computation may give it one (by transfering ownership or suballocation)
- `Bound`
	- The resource binding exists and the value may be written to by a queue, but no action has been taken to populate the bound region with a value.
- `Encoded`
	- The coordinator has prepared work that will populate the bound region with a value, but has not yet sent it to a queue.
- `Submitted`
	- The coordinator has sent work to a queue which will populate the bound region with a value, but it is not known on the local timeline whether that work has completed.
- `Ready`
	- The work has completed and there is now a value in the bound region.
- `Dead
	- There is no longer any resource associated to this slot and no computation may give it one

Notably, the `Submitted` and `Ready` states are the only states where the place the slot is associated with has any interaction with the computation.

### Buffer Allocators

Buffer allocators (sometimes abrieviated to just "buffers" in caiman) track the dynamic state of the bounds of unallocated regions of memory.  Allocation from them causes them to shrink.  A buffer may have a statically defined size, which caiman will track if the allocations are all statically sized.  Caiman also supports dynamic allocations, which may fail.  The resulting buffer cannot have a statically defined size, but may be used for further dynamic allocation (at the cost of needing to check for allocation failure).

A buffer carries a "spatial tag" that tracks which abstract resource it is associated with.  Allocated slots must also be marked with this same spatial tag.

There is currently no way to free memory.  It is intended that a future extension will use spatial tags to enforce safe suballocation, deallocation, and merging of buffers.

### Tags

Key to enforcing the invariant that a schedule cannot change the meaning of a value program (modulo termination) are three "tags" that track the (local) state of a slot, buffer, or fence in relation to an associated value funclet, timeline funclet, or spatial funclet.

A "value tag" associates a slot to a node or result of a value funclet.  The slot can only contain the result of that specific value node.

A "timeline tag" associates a slot or fence to a node in a timeline funclet when that slot or fence was last in a known state.  Each scheduling funclet also tracks a timeline tag for the state of the local queue itself.  This advances forward on a coordination event.

A "spatial tag" associates a slot or buffer to an abstract resource (a space).  These abstract resources carry no size or state, but constraints on the spatial funclet language should enforce that abstract operations (like splitting or merging) on associated spaces are safe. Caiman should (but doesn't always because of an incomplete implementation) enforce that all slots of the same space are discarded before the space is reused.

These correspond to the "what", "when", and "where" of computation, respectively.  Caiman enforces that the scheduling language respects the constraints inherited from the associated value, timeline, and spatial funclets.  Thus, analysis can be done on these funclets separately.

Abstractly, each of these (non-scheduling) funclets encodes a finite-state machine (a tree automaton in the case of the value language), slots are a subset of the product states of all three machines, and the scheduling funclet is choosing a linear path through a subset of the states where all three machines have a valid transition.  Thus, while most languages start with a subgraph of the product automaton and use static analysis to identify alternative paths (via optimizations), caiman is starting with all parts factored such that all paths are easily visible and is just checking the validity of the user-chosen path through the product automaton.

## The Core Scheduling Language

To do

### Control Flow

## The Timeline Language

To do

## The Spatial Language

To do

## Explication

To do

## Possibly related work

- [A cool webgpu binding thing that sparked work on caiman](https://github.com/Checkmate50/wgpu)
- [GRAMPS](https://graphics.stanford.edu/papers/gramps-tog/gramps-tog08.pdf)
- [Sequoia](https://graphics.stanford.edu/papers/sequoia/)
- [Legion](https://legion.stanford.edu)
- [@use-gpu/shader](https://acko.net/blog/frickin-shaders-with-frickin-laser-beams/) (A bit more of a stretch)
- [FrameGraph](https://www.gdcvault.com/play/1024612/FrameGraph-Extensible-Rendering-Architecture-in) (A bit less out there)
