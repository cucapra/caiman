# Caiman IR Design

Note: The following document describes things that might not yet be implemented in caimanc

## Motivation

Caiman is a programming language IR and compiler for rapidly exploring the design space of programs using both a CPU and GPU.  Portably programming for mixed GPU/CPU programs generally entails the use of DirectX, Vulkan, or Metal.  These API designs feature a common point of friction: CPU functions cannot directly invoke GPU functions.  Instead, the user must, manually, load a portable bytecode encoding the GPU function, construct a pipeline object that encodes all state necessary to complete the compilation, create buffers for staging data transfers between CPU and GPU, create binding objects (descriptors) to specify resources to be used by a pipeline, and create an encoding object to bind all elements together for submission to a queue.  These APIs also tend to expect manual programmer management and tracking of concurrent access hazards and lifetimes for resources in use by the CPU and GPU.  Metal and WebGPU provide stronger safety guarantees, but do so by heavily restricting the API's capability and inferring the optimal resource management at runtime using heuristics that the user has no control over.

In a GPGPU compute context, languages like CUDA or SYCL provide a single-source unified programming language that allows convenient code movement across the CPU and GPU boundary.  In these systems, functions on both the CPU and GPU can be analyzed by the same compiler to infer most of the boilerplate in a function call from CPU to GPU and code existing within a common subset of the language can migrate unchanged across the boundary.  This facilitates rapid development and exploration, but has a critical limitation: the similarities between the two sides of the boundary are syntactic but not semantic.  That is, a GPU function must access memory that is marked as accessible from the GPU and a CPU function must access memory that is marked as accessible from the CPU.  This, in general, reflects an underlying physical reality of the memory subsystem.  For data to be efficiently accessible on one, it must be physically copied a significant distance from the other.  This means the programmer must be aware of resource placement at all times.  In DirectX, Vulkan or Metal, the programmer must manually copy data between buffers.  CUDA's unified virtual memory and SYCL's unified sparse memory provide a convenient mechanism for passing pointers between CPU and GPU with minimal code changes, but at the cost of unpredictable latency as the underlying memory subsystem fetches the data.  This capability is non-portable and is generally undesirable for a low-latency environment.

The IR of Caiman is an attempt at a middle ground between the two approaches that borrows the philosophy of separation of scheduling concerns from the Halide image processing language.  By factoring out scheduling as a language, the IR facilitates the exploration of traditional compiler optimizations across the CPU/GPU boundary without silently violating the complicated rules of CPU/GPU boilerplate or hiding the underlying costs.

## Base Design

Most real-time rendering applications are designed around a render loop in which the visible scene must be updated and rerendered within the span of 7, 16, or sometimes 33 miliseconds (144 Hz, 60 Hz, or 30 Hz respectively).  Even for a slightly less-than-naïve implementation, this requires careful management of resource synchronization to avoid pipeline bubbles between the CPU and GPU.  For example, it is standard practice to begin issuing work to the GPU for the next frame while the prior frame's GPU work is still in the queue.  This entails management of several frames of resources and synchronization on points in time well before the most recently submitted work.  More sophisticated renderers must coordinate multiple render passes with resource uploads to the GPU, which may depend on reading information from the GPU to the CPU to know which resources must be uploaded.

Caiman's focus on the interface between the CPU and GPU means that it is mostly oriented around facilitating latency related optimizations and tight resource management at what is traditionally the graphics API layer executing on the CPU.  As such, as a near term goal, it is less concerned with decomposing or transforming shaders or compute kernels within a bulk GPU dispatch, unlike languages such as Halide, but is theoretically composable with such a system.  The intent is that one writes their shader code in a standard shading language (WGSL or any language that compiles to SPIRV) and uses Caiman to connect it with a host application in place of direct graphics API calls on the CPU.  It is also designed to allow progressive embedding in a hosting Rust and WebGPU application.  A programmer may reimplement only parts of their application in Caiman and interface it with their existing Rust and WebGPU code with minimal overhead.

In Caiman's IR, programs are split into two parts: a functional part, which specifies the transformation of values in a program, and a scheduling part, which specifies when and where the values are computed and stored.  The functional (value) part is, abstractly, a directed acyclic dataflow graph of pure functions (with multiple returns for some nodes like function calls).  Any logical dependency must appear as inputs to a functional node and all changes to state must appear as an output of the node.  Evaluation order is meaningless for functional nodes.  The scheduling part is, abstractly, a higher order function (a scheduling function) that takes a value function as input and produces a scheduled procedure, a linear sequence of side-effecting commands that implement functional nodes by operating on resources.  A scheduling function is therefore responsible for implementing the evaluation of functional nodes.  This separation ensures that a schedule can be completely removed or altered while trivially preserving the original functional specification.  A programmer can first define the functional behavior and then explore the design space of optimizations without fear of breaking the intended functionality.

This minimizes the false coupling of the underlying algorithms to a particular resource.  Functional nodes, when appearing to a scheduling function, may have particular requirements on the resource placement of their inputs or outputs.  For example, a compute kernel dispatch (which is necessarily scheduled on the GPU) can only receive data arguments that are resident on the GPU and can only output data that is resident on the GPU, but its dimension arguments must be resident on the CPU.  Meanwhile, a call to a CPU function (which is necessarily scheduled for execution on the CPU) can (currently) only input and output CPU resident data.  Using the output of one as the input of the other requires an explicit or implicit resource transition in the scheduling part, making visible and checkable a design trade-off of importance to performance-minded users.

At minimum, Caiman provides a guarantee that it will not accept an invalid schedule.  Scheduling functions can be statically verified to implement the provided functional nodes.  However given a partial specification of constraints, under most circumstances, at least one valid schedule will be practically computable in a schedule inference process called "explication".  This permits initial implementation of the functional part without implementation of the scheduling part and also reduces the boilerplate needed to .  Caiman defaults to a naïve, predictable schedule inference in cases of partial schedules so that a schedule with partial annotations is both sound and complete with respect to the language with full annotations.

## Pipelines and Memory Management

Caiman is primarily designed around two kinds of pipelines: oneshot pipelines, which execute and return control to the host application when done, and multi-stage (and potentially coinductive) pipelines, which return an opaque object with several possible re-entry points which may be invoked by the host application to decide control flow.  Oneshot pipelines are best thought of as standalone functions.  Multi-stage pipelines may be thought of as objects or coroutines.

The scheduling language is designed assuming that functions cannot allocate or deallocate resources such as buffers unless the maximum capacity is statically known.  This means that all pipelines are finite state machines.  For multi-stage pipelines, re-entry points represent state transitions invoked explicitly by the hosting environment.  Each method consumes the previous pipeline object and produces a new pipeline object.  Methods may receive borrowed references to external resources as input or output arguments.  These may be accessed through special variables known as "slots" (discussed below and in the IR section).

## Caiman IR design

Caiman's IR organizes programs into pipelines (functions and object constructors) that connect funclets (basic blocks).  The operations and nodes in a funclet are encoded as an array where each entry is a node or operation that can only depend on nodes with an index less than its own.  The tail edge (terminator node) for the funclet is encoded separately from the array and captures control flow (if control flow is supported for the relevant language).  Currently, the scheduling part of each basic block (a funclet) can be encoded inline with this array, but inline functional nodes cannot depend on scheduling nodes.  Functional nodes that appear in this array in a mixed language context unambiguously map to a CPU computation on the coordinator that schedules or computes the node on the appropriate queue (via `encode_do` or `encode_copy`).  The implicit ordering and scheduling associated with this array makes it easy to unambiguously imply a specific naive schedule by only specifying functional nodes.  There is strong intent to at least allow separation of the scheduling part from the value part and possibly require a full separation.  This will likely come in a later refactoring.  The scheduling language as described below reflects the intended scheduling language after this split.

The name "funclet" is shamelessly taken from https://github.com/WebAssembly/funclets/blob/main/proposals/funclets/Overview.md
The structure of funclets is loosely similar to cranelift (https://github.com/bytecodealliance/wasmtime/blob/main/cranelift/docs/ir.md)

### Defining pipelines

To do

## The (Value) Functional Language

To do

## The Schedule Execution Model

A scheduling function can only distiguish nodes by their dependencies and placement requirements, so, for example, any binary operation requiring all resources to be on the CPU will behave identically as far as the schedule is concerned.

### Queues

Much like the GPU APIs it is built on, Caiman adopts the in-order queue as an organizing principle.  A system may contain multiple threads of execution with different capabilities and each is referred to as a "place".  Each place has a single queue and all work must appear to complete in FIFO order.  Queues coordinate through fences which signal asynchronously to notify other queues of completion.  This may be extended in the future to allow more sophisticated synchronization patterns on more realistic work execution models.

Caiman programs are executed from the perspective of a (usually) single, centralized Coordinator responsible for encoding and issuing commands to other queues.  The Coordinator logically executes in the "Local" queue, which (in the current implementation) corresponds to the calling thread of execution of the host program.  As such, the Coordinator may (currently) be reasoned about in terms of performance as executing on the CPU.  However, this model will likely need to change to support indirect work dispatch on the GPU and parallel command encoding.

### Time and Operations

All operations in schedules are associated with a monotonically increasing logical timestamp for the local queue.  Additionally, all operations are functions of the whole local state and may therefore modify type information of values not directly output as a return value.

### Slots

Slots are the key piece of state in schedules.  They do not hold values themselves.  Instead, they are references that associate a subexpression of an invocation of a value function to the corresponding subregion of a resource expected to hold that value at some point in the future.

The associated value subexpression may be seen as one part of the type of the slot.  Additionally, slots have type information that tracks the state of the value as known to the coordinator which change as a result of operations that modify the Coordinator's timeline.  This includes a logical time stamp for when the coordinator last observed a state transition for that slot, a tag for the queue that is expected to produce the value, and a tag encoding what are (currently) 4 mutually exclusive states:

- `None`
	- The resource binding exists and the value may be written to by a queue, but no action has been taken to populate the bound region with a value.
- `Encoded`
	- The Coordinator has prepared work that will populate the bound region with a value, but has not yet sent it to a queue.
- `Submitted`
	- The Coordinator has sent work to a queue which will populate the bound region with a value, but it is not known on the Local timeline whether that work has completed.
- `Ready`
	- The work has completed and there is now a value in the bound region.

## The Core Scheduling Language

The scheduling language provides the following operations in almost all contexts.

- `alloc_temporary (value_tag : (InstanceId, SubexpressionId), offset : usize, size : usize) -> SlotId`
	- immediately creates a slot with future value `value_tag` in the `None` state that is bound to the region of the local temporary buffer at `offset` and ending at `offset + size` (exclusive)
	- This range cannot overlap the range of another slot
- `bind_buffer (value_tag : (InstanceId, SubexpressionId), buffer_id : BufferId, offset : usize, size : usize) -> SlotId`
	- immediately creates a slot with future value `value_tag` in the `None` state that is bound to the region of `buffer_id` starting at `offset` and ending at `offset + size` (exclusive)
	- This range cannot overlap the range of another slot
- `encode_copy (place : Place, from_slot_id : SlotId, to_slot_id : SlotId)`
	- schedules to `place` the copy of the memory from `from_slot_id` to `to_slot_id`
	- `from_slot_id` must at least be in the `Encoded` state and have a resource binding
	- `to_slot_id` must be in the `None` state and have a resource binding with equal size to that of `from_slot_id`
	- The value part of both slots must match
- `encode_do (place : Place, value_tag : (InstanceId, SubexpressionId),  input_slots : [SlotId], output_slots : [SlotId])`
	- schedules to `place` the execution of the subexpression specified by `value_tag` with inputs bound to `input_slots` and outputs bound to `output_slots`
	- For most value nodes, `value_tag` will also be the type of the (usually one) slot in `output_slots` and could theoretically be implied by the value part of the type of the output slot.  The reason this parameter is necessary at all is for nodes with multiple returns, which are modeled as returning a fake tuple of outputs (and is therefore not a real node with resource needs).  Each component of the return value in such a case is then a separate node in the value graph.
	- Since output memory is usually allocated ahead of time and passed by reference into the called function, the various outputs of function calls are not stored adjacently in memory as its pseudo-tuple type might suggest.  Therefore `encode_do` needs all inputs and outputs specified upfront so codegen can emit code correctly without implicitly allocating and copying or predicting the future.
- `encode_map_read_only (place : Place, mapped_slot : SlotId) -> SlotId`
	- schedules to `place` the creation of a slot containing a read-only reference to a bound buffer region referenced by `mapped_slot`
	- the created slot will have the same value as `mapped_slot`
	- `mapped_slot` must be in the `Ready` state
	- No writeable mapping may be allowed at the same time as a readable mapping
- `encode_map_write_only (place : Place, mapped_slot : SlotId) -> SlotId`
	- schedules to `place` the creation of a slot containing a write-only reference to a bound buffer region referenced by `mapped_slot`
	- the created slot will have the same value as `mapped_slot`
	- No other mapping (read only or writeable) may be allowed at any given time and `mapped_slot` must be in the `None` state
	- Upon encoding a value to this slot, `mapped_slot` will also be in the `Encoded` state and the created slot will be immediately `discard`ed
- `encode_forward (value_tag : (InstanceId, SubexpressionId), forwarded_slot_id : SlotId) -> SlotId`
	- schedules to `place` the creation of a slot with value specified as `value_tag` which will have its resource bindings transfered from `forwarded_slot_id` simultaneously with the next read of `forwarded_slot_id`
	- This is useful for specifying in-place mutation
	- This slot does not technically exist, and so may not be used, until the next read of `forwarded_slot_id`, whereupon `forwarded_slot_id` will be `discard`ed
- `submit (place : Place)`
	- submits the pending buffer of encoded commands scheduled for `place` and increments the logical timestamp
- `encode_fence (place : Place) -> FenceId`
	- encodes a fence to be signaled when the specified place reaches this point in its execution
- `encode_sync_fence (place : Place, fence_id : FenceId)`
	- encodes a synchronization of `place` on the fence specified by `fence_id`.  No work will be executed on place until the queue for which the fence was encoded has progressed past the fence
	- This fence cannot be a fence created in the future
	- This operation increments the logical timestamp and updates the queue type state of all relevant slots
- `discard (slots : [SlotId])`
	- erases the given slots and unmaps any associated resources, if applicable

Leaf (input) nodes of a value function behave in a special way when read.  Rather than requiring the value part of the type (`(InstanceId, SubexpressionId)`) to be the same, which would prevent instantiation of multiple functions that read the same values from the same bindings, the value check for leaf nodes is satisfied if the type of the value of the slot matches the type of the leaf node.

Offsets provided for some hardware may be illegal for others based on alignment.

Under webgpu, reads or writes (`encode_map_*`) from or to slots bound to a buffer may force an implicit synchronization if any slot bound to that buffer may be in use by the GPU.  This is an unfortunate possibility handed down from webgpu that would be unnecessary on most other APIs since caiman statically enforces the constraint at a finer granularity.

Encoding to the local queue immediately evaluates the relevant node and transitions slots directly to `Ready` instead of `Encoded` since the Coordinator is always synchronized with the local queue.

### Terminators and Control Flow

To do

## Explication

The above scheduling language is (partly) designed to permit partial schedules, where some parameters may be left unspecified (written in this document as the expression `?`) and (most) scheduling operations may be omitted (with holes explicitly written in this document as the pseudo-operation `???`).  The intended goal for this capability is to permit more compact partial schedules that unambiguously imply a full schedule, but it also allows expansion of the expressivity of the scheduling language while (generally) maintaining compatibility and (mostly) allowing schedule equivalence with full or partial schedules written for a less expressive scheduling language.

The process of inferring a schedule from a partially given one is called "schedule explication" as it makes the schedule explicit for the purposes of the codegen.  The part of the compiler that does "schedule explication" is called the "explicator".

A couple points of the scheduling language facilitate explication:
- The queue states form a total ordering that unambiguously implies a path for the explicator to generate when the schedule is left implicit.
	- The canonical path is `None` -> `Encoded` via `encode_copy` (if the data already exists) or `encode_do` scheduled immediately (necessary to preserve ordering), `Encoded` -> `Submitted` via `submit` as late as legally possible, `Submitted` -> `Ready` via synchronization as late as possible on a fence inserted as late as possible.  Each of the inserted operations is performed as late as possible to be  manually written operations to make them unnecessary.
- `discard` is never inserted
- `alloc_temporary` and `bind_buffer` always bump allocate and never fill holes, even if memory in lower regions has been made available via discard
	- Additionally, `bind_buffer` will always allocate from a buffer that is visible only to the explicator (one for each place)
- slots arguments may be left unspecified if there is only one slot (in the full set of slots visible to the coordinator) for the needed queue and value

Additional scheduling operations that are meaningful only to the explicator can act as hints:
- `hide(slots : [SlotId])` / `show(slots : [SlotId])`
	- removes/adds a slot from/to consideration by the explicator to avoid ambiguity, but will still be considered when checking for errors
- `pin (slots : [SlotId]) -> Pin` / `unpin (pin : Pin, slots : [SlotId])`
	- requires that all queue state transitions for these nodes be done before the pin or after the corresponding unpin
- `???`
	- Allows the explicator to insert operations here
	- This is probably more predictable and preferable over a more permissive always-explicate model in the event that the scheduling language stops being strictly schedule equivalent with language subsets

## Interfacing with the host

To do

## Possibly related work

- [](https://github.com/Checkmate50/wgpu)
- [GRAMPS](https://graphics.stanford.edu/papers/gramps-tog/gramps-tog08.pdf)
- [Sequoia](https://graphics.stanford.edu/papers/sequoia/)
- [Legion](https://legion.stanford.edu)
- [@use-gpu/shader](https://acko.net/blog/frickin-shaders-with-frickin-laser-beams/) (A bit more of a stretch)
- [FrameGraph](https://www.gdcvault.com/play/1024612/FrameGraph-Extensible-Rendering-Architecture-in) (A bit less out there)
- Your favorite reimplementation of DX11 colloquially known as a HAL goes here


To do: Describe the scheduler state more formally and what a variable is in the scheduling language

To do: More on why state machines are (probably) the right model and how control flow works in Caiman

To do: More on where expanded control flow would make sense

To do: Actual comparison

To do: Comment (rant) on problems in opengl, vulkan/dx12, and webgpu motivating parts of caiman's design.  Also, more on why just copying the CUDA model cannot fix it.
