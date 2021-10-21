use crate::ir;
use crate::analyses;

mod split_cpu_gpu
{
	use crate::ir;
	use crate::analyses;
	
	// Compute node accessibility
	// Identify all callexternal nodes where accessibility does not meet requirements and legalize them with callgpucoordinator nodes
	fn split_funclet(funclet : &ir::Funclet)
	{
	}
}