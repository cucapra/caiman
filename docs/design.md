# Caiman IR Design

## Motivation

Caiman is a programming language IR and compiler for rapidly exploring the design space of programs using both a CPU and GPU.  Portably programming for mixed GPU/CPU programs generally entails the use of DirectX, Vulkan, or Metal.  These API designs feature a common point of friction: CPU functions cannot directly invoke GPU functions.  Instead, the user must, manually, load a portable bytecode encoding the GPU function, construct a pipeline object that encodes all state necessary to complete the compilation, create buffers for staging data transfers between CPU and GPU, create binding objects (descriptors) to specify resources to be used by a pipeline, and create an encoding object to bind all elements together for submission to a queue.  These APIs also tend to expect manual programmer management and tracking of concurrent access hazards and lifetimes for resources in use by the CPU and GPU.  Metal and WebGPU provide stronger safety guarantees, but do so by heavily restricting the API's capability and inferring the optimal resource management at runtime using heuristics that the user has no control over.

In a GPGPU compute context, languages like CUDA or SYCL provide a single-source unified programming language that allows convenient code movement across the CPU and GPU boundary.  In these systems, functions on both the CPU and GPU can be analyzed by the same compiler to infer most of the boilerplate in a function call from CPU to GPU and code existing within a common subset of the language can migrate unchanged across the boundary.  This facilitates rapid development and exploration, but has a critical limitation: the similarities between the two sides of the boundary are  syntactic but not semantic.  That is, a GPU function must access memory that is marked as accessible from the GPU and a CPU function must access memory that is marked as accessible from the CPU.  This, in general, reflects an underlying physical reality of the memory subsystem.  For data to be efficiently accessible on one, it must be physically copied a significant distance.  This means the programmer must be aware of resource placement at all times.  In DirectX, Vulkan or Metal, the programmer must manually copy data between buffers.  CUDA's unified virtual memory and SYCL's unified sparse memory provide a convenient mechanism for passing pointers between CPU and GPU with minimal code changes, but at the cost of unpredictable latency as the underlying memory subsystem fetches the data.  This capability is non-portable and is generally undesirable for a low-latency environment.

The IR of Caiman is an attempt at a middle ground between the two approaches that borrows the philosophy of separation of scheduling concerns from the Halide image processing language.  By factoring out the scheduling language, the IR facilitates the exploration of traditional compiler optimizations across the CPU/GPU boundary without silently violating the complicated rules of CPU/GPU boilerplate or hiding the underlying costs.

## Base Design

In Caiman's IR, programs are split into two parts: a functional part, which specifies the transformation of values in a program, and a scheduling part, which specifies where the values are computed and stored.

The functional part is, abstractly, a value dependence graph.  There can be no side-effects for any nodes.  Any logical dependency must appear as inputs to a functional node and all changes to state must appear as an output.  This minimizes the false coupling of the underlying algorithms to a particular resource.  Functional nodes may have particular requirements on the resource placement of their inputs or outputs.  For example, a compute kernel dispatch (which is necessarily scheduled on the GPU) can only receive data arguments that are resident on the GPU and can only output data that is resident on the GPU, but its dimension arguments must be resident on the CPU.  Meanwhile, a call to a CPU function (which is necessarily scheduled locally) can (currently) only input and output CPU resident data.  Using the output of one as the input of the other requires an explicit or implicit resource transition in the scheduling part.

The scheduling part is, abstractly, a function that takes the value dependence graph as input and produces a schedule (a linear sequence of commands that implement functional nodes using resources that contain values).  The schedule it outputs must respect a topological ordering on the functional graph.  All schedules must be checkable to enforce this invariant.  If the functional part has a valid schedule but no schedule specified, at least one valid schedule must be practically computable.

A key design goal is that the schedule part should always be optional yet predictable when specified.

This separation ensures that a schedule can be completely removed or altered while trivially preserving the original functional specification.  A programmer can first define the functional behavior and then explore the design space of optimizations without fear of breaking the intended functionality.

Caiman's IR adopts a centralized execution model.  There is a single, central Coordinator that is responsible for encoding and issuing commands to a local queue or a GPU queue.  Caiman currently uses a simple scheduling model where work will appear to complete in a FIFO order and the GPU queue cannot create local queue work.  The restrictions on the GPU queue currently correspond to limitations of WebGPU.  The restrictions on the local (unfortunately conflated with CPU) queue may be alleviated by introducing a more sophisticated scheduling language.

Caiman's IR organizes programs into pipelines (functions) that connect funclets (basic blocks).  The functional part of each funclet is encoded as an array where each entry is a node that can only depend on other functional nodes with an index less than its own.  The tail edge (terminator node) for the funclet is encoded separately from the array and captures control flow.  The scheduling part of each basic block (a funclet) is encoded inline with this array.  Functional nodes that appear in this array unambiguously map to a CPU computation on the coordinator that schedules or computes the node on the appropriate queue (via `do_local` or `encode_gpu`).  The implicit ordering and scheduling associated with this array makes it easy to unambiguously imply a specific naive schedule by only specifying functional nodes.  This allows the compiler to more directly expose the optimization process to the user and minimize internal heuristics.

The process of inferring a schedule from a partially given one is called "schedule explication" as it makes the schedule explicit for the purposes of the codegen.  The part of the compiler that does "schedule explication" is called the "explicator".

## The Schedule Execution Model

Each functional node has an entry in a dictionary tracking the coordinator's last known state of each value on each queue. On the local queue, a value may either be resident or not.  On the GPU queue, a value may be nonresident, encoded, submitted, or completed.  The GPU value states form an ordering that unambiguously implies a path for the explicator to generate when the schedule is left implicit.  These states will likely be expanded.  This dictionary is encapsulated in a resource structure (the resource state) that also encodes the state of paired instructions like pin/unpin.

Scheduling "nodes" are functions that operate directly on the resource state in order of appearance in the array and should not be thought of as part of the value dependence graph (they can have no output that can be depended on by a functional node).  They cannot currently have dependencies on runtime-dynamic values.

Scheduling nodes (some are not yet implemented):

- `do_local (node : NodeId)`
	- Implicitly inserted for CPU-only functional nodes in the array representation
	- Executes a given functional node in the local queue (synchronously)
- `encode_gpu (nodes : [NodeId])`
	- Implicitly inserted for GPU-only functional nodes in the array representation
	- Synchronously encodes a given functional node to the GPU queue but does not yet submit for asynchronous completion
	- If the node in question has already completed on the CPU, it schedules a copy of the data
- `submit_gpu (nodes : [NodeId])`
	- Synchronously enqueues the given encoded nodes to the GPU for asynchronous completion.
	- With WebGPU, this requires any node currently in encoding to be submitted.
- `sync_local (nodes : [NodeId])`
	- Synchronously awaits completion of a GPU node and makes result locally resident.
- `pin (nodes : [NodeId])` / `unpin (pin, nodes : [NodeId])`
	- requires that all resource transitions for these nodes be done before the pin or after the corresponding unpin
- `begin_once` / `end_once(begin)`
	- Declares a section that may not use nodes dependent on input functional nodes
	- Subschedule will be executed only once
- `begin_explicit` / `end_explicit(begin)`
	- Declarates a section that is assumed to be explicitly scheduled.  The explicator will only check that the subschedule is legal.

