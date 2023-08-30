@group(0) @binding(0) var<uniform> dimensions : vec2<u32>;
@group(0) @binding(1) var<storage> matrix : array<f32>;
@group(0) @binding(2) var<storage> input_vector : array<f32>;
@group(0) @binding(3) var<storage, read_write> output_vector : array<f32>;

// A very dumb implementation of matrix-vector multiplication on the GPU
@compute @workgroup_size(32) 
fn matvecmul(@builtin(local_invocation_index) local_invocation_index : u32, @builtin(num_workgroups) num_workgroups : vec3<u32>)
{
	var value : f32 = 0.0;
	var rows : u32 = dimensions.y;
	for (var i : u32 = 0u; i < rows; i++)
	{
		value += input_vector[i] * matrix[i + local_invocation_index * dimensions.y];
	}
	output_vector[local_invocation_index] = value;
}
