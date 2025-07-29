// Type #8: GpuFence
// Type #9: GpuEncoder
// Type #0: I64
// Type #1: Array { element_type: TypeId(7), length: 2 }
// Type #2: Array { element_type: TypeId(7), length: 4 }
// Type #3: Array { element_type: TypeId(7), length: 8 }
// Type #4: Array { element_type: TypeId(0), length: 2 }
// Type #5: Array { element_type: TypeId(0), length: 4 }
// Type #6: Array { element_type: TypeId(0), length: 8 }
// Type #7: I32
// Type #10: USize
// Type #11: U64

		/*pub struct CpuFunctionInvocationState<'parent>
		{
			parent_state : & 'parent mut dyn caiman_rt::State
		}*/
#[allow(warnings)] pub mod main {
use caiman_rt::{LocalVars, GpuLocals, wgpu, bytemuck};
use std::marker::PhantomData;
#[allow(warnings)] pub mod outputs {
pub type _lt_i64_i64 = (i32, );
pub type sum = (i64, );
}
pub trait CpuFunctions
{
	fn _lt_i64_i64(&self, state : &mut caiman_rt::State, _ : i64, _ : i64) -> outputs::_lt_i64_i64;
	fn sum(&self, state : &mut caiman_rt::State, _ : [i64; 4]) -> outputs::sum;
}
fn funclet4_func<'state,  'cpu_functions, 'callee, Callbacks : CpuFunctions>(mut instance : Instance<'state, 'cpu_functions, Callbacks>, join_stack : &mut caiman_rt::JoinStack<'callee>, var_0 : [i64; 4], var_1 : [i64; 4], var_2 : [i64; 4] ) -> FuncletResult<'state, 'cpu_functions, 'callee, Callbacks, PipelineOutputTuple<'callee>>
{
	use std::convert::TryInto;
//  node #0: Phi { index: 0 }
//  node #1: Phi { index: 1 }
//  node #2: Phi { index: 2 }
//  node #3: AllocTemporary { place: Local, storage_type: TypeId(0), buffer_flags: BufferFlags { map_read: true, map_write: true, copy_src: true, copy_dst: true, storage: false, uniform: false } }
instance.locals.malloc::<i64>(3);
//  node #4: LocalDoExternal { operation: Node { node_id: 3 }, external_function_id: ExternalFunctionId(2), inputs: [0], outputs: [3] }
let var_4 = instance.cpu_functions.sum(instance.state, var_0);
*instance.locals.calloc::<i64>(5, var_4.0);
*instance.locals.get_mut::<i64>(3) = (*instance.locals.get::<i64>(5));
//  node #5: ReadRef { storage_type: TypeId(0), source: 3 }
let var_6 = (*instance.locals.get::<i64>(3));
instance.locals.calloc::<i64>(6, var_6);
//  node #6: AllocTemporary { place: Local, storage_type: TypeId(0), buffer_flags: BufferFlags { map_read: true, map_write: true, copy_src: true, copy_dst: true, storage: false, uniform: false } }
instance.locals.malloc::<i64>(7);
//  node #7: LocalDoBuiltin { operation: Node { node_id: 9 }, inputs: [], outputs: [6] }
instance.locals.calloc::<i64>(8, 0);
*instance.locals.get_mut::<i64>(7) = (*instance.locals.get::<i64>(8));
//  node #8: ReadRef { storage_type: TypeId(0), source: 6 }
let var_9 = (*instance.locals.get::<i64>(7));
instance.locals.calloc::<i64>(9, var_9);
//  node #9: AllocTemporary { place: Local, storage_type: TypeId(7), buffer_flags: BufferFlags { map_read: true, map_write: true, copy_src: true, copy_dst: true, storage: false, uniform: false } }
instance.locals.malloc::<i32>(10);
//  node #10: LocalDoExternal { operation: Node { node_id: 10 }, external_function_id: ExternalFunctionId(1), inputs: [5, 8], outputs: [9] }
let var_11 = instance.cpu_functions._lt_i64_i64(instance.state, var_6, var_9);
*instance.locals.calloc::<i32>(12, var_11.0);
*instance.locals.get_mut::<i32>(10) = (*instance.locals.get::<i32>(12));
//  node #11: ReadRef { storage_type: TypeId(7), source: 9 }
let var_13 = (*instance.locals.get::<i32>(10));
instance.locals.calloc::<i32>(13, var_13);
//  node #12: DefaultJoin
//  node #13: InlineJoin { funclet: 5, captures: [], continuation: 12 }
//  tail edge: ScheduleSelect { value_operation: Node { node_id: 12 }, timeline_operation: None, spatial_operation: None, condition: 11, callee_funclet_ids: [6, 7], callee_arguments: [1, 2], continuation_join: 13 }
if var_13 !=0 { //  node #0: Phi { index: 0 }
//  node #1: Phi { index: 1 }
//  node #2: AllocTemporary { place: Local, storage_type: TypeId(0), buffer_flags: BufferFlags { map_read: true, map_write: true, copy_src: true, copy_dst: true, storage: false, uniform: false } }
instance.locals.malloc::<i64>(15);
//  node #3: LocalDoExternal { operation: Node { node_id: 5 }, external_function_id: ExternalFunctionId(2), inputs: [0], outputs: [2] }
let var_16 = instance.cpu_functions.sum(instance.state, var_1);
*instance.locals.calloc::<i64>(17, var_16.0);
*instance.locals.get_mut::<i64>(15) = (*instance.locals.get::<i64>(17));
//  node #4: ReadRef { storage_type: TypeId(0), source: 2 }
let var_18 = (*instance.locals.get::<i64>(15));
instance.locals.calloc::<i64>(18, var_18);
//  tail edge: Return { return_values: [4] }
if join_stack.used_bytes().len() > 0 { return pop_join_and_dispatch_at_0::<Callbacks, PipelineOutputTuple<'callee>>(join_stack, var_18, instance) }return FuncletResult::<'state, 'cpu_functions, 'callee, Callbacks, _> {phantom : std::marker::PhantomData::<& 'callee ()>, intermediates : FuncletResultIntermediates::<_>::Return((var_18, )), instance}; } else { //  node #0: Phi { index: 0 }
//  node #1: Phi { index: 1 }
//  node #2: AllocTemporary { place: Local, storage_type: TypeId(0), buffer_flags: BufferFlags { map_read: true, map_write: true, copy_src: true, copy_dst: true, storage: false, uniform: false } }
instance.locals.malloc::<i64>(19);
//  node #3: LocalDoExternal { operation: Node { node_id: 7 }, external_function_id: ExternalFunctionId(2), inputs: [1], outputs: [2] }
let var_20 = instance.cpu_functions.sum(instance.state, var_2);
*instance.locals.calloc::<i64>(21, var_20.0);
*instance.locals.get_mut::<i64>(19) = (*instance.locals.get::<i64>(21));
//  node #4: ReadRef { storage_type: TypeId(0), source: 2 }
let var_22 = (*instance.locals.get::<i64>(19));
instance.locals.calloc::<i64>(22, var_22);
//  tail edge: Return { return_values: [4] }
if join_stack.used_bytes().len() > 0 { return pop_join_and_dispatch_at_0::<Callbacks, PipelineOutputTuple<'callee>>(join_stack, var_22, instance) }return FuncletResult::<'state, 'cpu_functions, 'callee, Callbacks, _> {phantom : std::marker::PhantomData::<& 'callee ()>, intermediates : FuncletResultIntermediates::<_>::Return((var_22, )), instance}; }
}

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

		type PipelineOutputTuple<'callee> = (i64, );
enum FuncletResultIntermediates<Intermediates>
{ Return(Intermediates), Yield0{ yielded : () }, }impl<'state, 'cpu_functions, 'callee, Callbacks : CpuFunctions, Intermediates> FuncletResult<'state, 'cpu_functions, 'callee, Callbacks, Intermediates>
{
pub fn yielded_at__loop_impl(&self) -> Option<& ()> { if let FuncletResultIntermediates::Yield0{yielded} = & self.intermediates { Some(yielded) } else { None } }
}pub struct Instance<'state, 'cpu_functions, F : CpuFunctions>{state : & 'state mut dyn caiman_rt::State, cpu_functions : & 'cpu_functions F, locals: LocalVars, glocals: GpuLocals}

		impl<'state, 'cpu_functions, F : CpuFunctions> Instance<'state, 'cpu_functions, F>
		{
			pub fn new(state : & 'state mut dyn caiman_rt::State, cpu_functions : & 'cpu_functions F) -> Self
			{
				
				Self{locals: LocalVars::new(), glocals: GpuLocals::new(state), state, cpu_functions}
			}

		
		}
		impl<'state, 'cpu_functions, F : CpuFunctions> Instance<'state, 'cpu_functions, F>
{
pub fn start<'callee>(mut self, join_stack : &mut caiman_rt::JoinStack<'callee>, arg_0 : [i64; 4], arg_1 : [i64; 4], arg_2 : [i64; 4]) -> FuncletResult<'state, 'cpu_functions, 'callee, F, PipelineOutputTuple<'callee>> {
let r = funclet4_func(self, join_stack, arg_0, arg_1, arg_2);
r }pub fn resume_at__loop_impl<'callee>(self, join_stack : &mut caiman_rt::JoinStack<'callee>) -> FuncletResult<'state, 'cpu_functions, 'callee, F, PipelineOutputTuple<'callee>> { pop_join_and_dispatch_at_1::<F, PipelineOutputTuple<'callee>>(join_stack, self) }
}
#[derive(Debug)] enum ClosureHeader { Root, }
fn pop_join_and_dispatch_at_0<'state, 'cpu_functions, 'callee, Callbacks : CpuFunctions, Intermediates>(join_stack : &mut caiman_rt::JoinStack<'callee>, arg_0 : i64, mut instance : Instance<'state, 'cpu_functions, Callbacks> ) -> FuncletResult<'state, 'cpu_functions, 'callee, Callbacks, (i64, )>
{
let closure_header = unsafe { join_stack.pop_unsafe_unaligned::<ClosureHeader>().unwrap() }; match closure_header {
_ => panic!("Dispatcher cannot dispatch given closure {:?}", closure_header), } }fn pop_join_and_dispatch_at_1<'state, 'cpu_functions, 'callee, Callbacks : CpuFunctions, Intermediates>(join_stack : &mut caiman_rt::JoinStack<'callee>, mut instance : Instance<'state, 'cpu_functions, Callbacks> ) -> FuncletResult<'state, 'cpu_functions, 'callee, Callbacks, (i64, )>
{
let closure_header = unsafe { join_stack.pop_unsafe_unaligned::<ClosureHeader>().unwrap() }; match closure_header {
_ => panic!("Dispatcher cannot dispatch given closure {:?}", closure_header), } }
        #[derive(Debug, Copy, Clone)]
        enum StackRef<T> {
            Local(usize, PhantomData<T>),
        }
        impl<T: 'static> StackRef<T> {
            fn get<'state, 'cpu_functions, 'a, F: CpuFunctions>(
                &self,
                instance: &'a Instance<'state, 'cpu_functions, F>,
            ) -> &'a T {
                match self {
                    StackRef::Local(index, ..) => instance.locals.get::<T>(*index),
                }
            }
    
            fn get_mut<'state, 'cpu_functions, 'a, F: CpuFunctions>(
                &self,
                instance: &'a mut Instance<'state, 'cpu_functions, F>,
            ) -> &'a mut T {
                match self {
                    StackRef::Local(index, ..) => instance.locals.get_mut::<T>(*index),
                }
            }
    
            fn local(index: usize) -> Self {
                StackRef::Local(index, PhantomData)
            }
        }}

