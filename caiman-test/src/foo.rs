// Type #0: I32
// Type #1: USize
// Type #2: U64

		use caiman_rt::wgpu;

		/*pub struct CpuFunctionInvocationState<'parent>
		{
			parent_state : & 'parent mut dyn caiman_rt::State
		}*/
pub mod main {
use super::*;
pub mod outputs {
}
pub trait CpuFunctions
{
}
fn funclet1_func<'state,  'cpu_functions, 'callee, Callbacks : CpuFunctions>(instance : Instance<'state, 'cpu_functions, Callbacks>, join_stack : &mut caiman_rt::JoinStack<'callee> ) -> FuncletResult<'state, 'cpu_functions, 'callee, Callbacks, (i32, )>
{
	use std::convert::TryInto;
//  node #0: AllocTemporary { place: Local, storage_type: TypeId(0), operation: RemoteNodeId { funclet_id: 0, node_id: 0 } }
//  node #1: EncodeDo { place: Local, operation: RemoteNodeId { funclet_id: 0, node_id: 0 }, inputs: [], outputs: [0] }
let var_0 : i32 = 5;
//  tail edge: Return { return_values: [0] }
if join_stack.used_bytes().len() > 0 { return pop_join_and_dispatch_at_0::<Callbacks, PipelineOutputTuple<'callee>>(instance, join_stack, var_0) }return FuncletResult::<'state, 'cpu_functions, 'callee, Callbacks, _> {instance, phantom : std::marker::PhantomData::<& 'callee ()>, intermediates : FuncletResultIntermediates::<_>::Return((var_0, ))};}

		pub struct FuncletResult<'state, 'cpu_functions, 'callee, Callbacks : CpuFunctions, Intermediates>
		{
			instance : Instance<'state, 'cpu_functions, Callbacks>,
			phantom : std::marker::PhantomData<& 'callee ()>,
			intermediates : FuncletResultIntermediates<Intermediates>
		}

		impl<'state, 'cpu_functions, 'callee, Callbacks : CpuFunctions, Intermediates> FuncletResult<'state, 'cpu_functions, 'callee, Callbacks, Intermediates>
		{
			pub fn returned(&self) -> Option<& Intermediates>
			{
				if let FuncletResultIntermediates::Return(intermediates) = & self.intermediates
				{
					return Some(& intermediates);
				}

				None
			}

			pub fn prepare_next(self) -> Instance<'state, 'cpu_functions, Callbacks>
			{
				self.instance
			}
		}

		type PipelineOutputTuple<'callee> = (i32, );
enum FuncletResultIntermediates<Intermediates>
{ Return(Intermediates), }impl<'state, 'cpu_functions, 'callee, Callbacks : CpuFunctions, Intermediates> FuncletResult<'state, 'cpu_functions, 'callee, Callbacks, Intermediates>
{
}pub struct Instance<'state, 'cpu_functions, F : CpuFunctions>{state : & 'state mut dyn caiman_rt::State, cpu_functions : & 'cpu_functions F}

		impl<'state, 'cpu_functions, F : CpuFunctions> Instance<'state, 'cpu_functions, F>
		{
			pub fn new(state : & 'state mut dyn caiman_rt::State, cpu_functions : & 'cpu_functions F) -> Self
			{
				
				Self{state, cpu_functions}
			}

		
		}
		impl<'state, 'cpu_functions, F : CpuFunctions> Instance<'state, 'cpu_functions, F>
{
pub fn start<'callee>(self, join_stack : &mut caiman_rt::JoinStack<'callee>) -> FuncletResult<'state, 'cpu_functions, 'callee, F, PipelineOutputTuple<'callee>> { funclet1_func(self, join_stack) }}
#[derive(Debug)] enum ClosureHeader { Root, }
fn pop_join_and_dispatch_at_0<'state, 'cpu_functions, 'callee, Callbacks : CpuFunctions, Intermediates>(instance : Instance<'state, 'cpu_functions, Callbacks>, join_stack : &mut caiman_rt::JoinStack<'callee>, arg_0 : i32 ) -> FuncletResult<'state, 'cpu_functions, 'callee, Callbacks, (i32, )>
{
let closure_header = unsafe { join_stack.pop_unsafe_unaligned::<ClosureHeader>().unwrap() }; match closure_header {
_ => panic!("Dispatcher cannot dispatch given closure {:?}", closure_header), } }}
