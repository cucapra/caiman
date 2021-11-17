struct Image
{

}

fn draw_triangle(vertices : &[[f32; 3]; 3], output_image : &mut Image)
{
	let future = async_gpu_coordinator!
	{
		let mut triangle_count = 0;

		// Auto detect variables from enclosing scope and generate the buffer for them with implicit copies/synchronization?
		dispatch!
		{
			dimensions: [1, 1, 1],
			arguments: bind!{ triangle_count: &mut triangle_count },
			compute_kernel: """
			void main()
			{
				triangle_count = 1;
			}
			"""
		};

		/*
		The above does:
		- creates a compute pipeline object (and layout)
		(The above should be done only once)
		- creates a command buffer
		- creates a buffer for storing the GPU result of triangle_count
		- creates bindgroups for the outputs
		- enqueues a write to the buffer of the existing value of triangle_count
		- binds the bindgroup containing a reference to the triangle_count buffer
		- binds the pipeline object
		- encodes a dispatch
		- submits it
		- awaits completion
		- reads the triangle_count buffer
		*/

		draw!
		{
			element_count: triangle_count,
			in_arguments: bind!{ vertices : & vertices, triangle_count: triangle_count },
			inout_arguments: bind! {},
			out_arguments: bind!{ output_image : &mut output_image },
			vertex_shader: """
			out vec3 color;
			void main()
			{
				int triangle_index = gl_VertexIndex / 3;
				float percentage = float(triangle_index + 1) / float(triangle_count);
				gl_Position = vec4(vertices[gl_VertexIndex % 3] * vec2(percentage), percentage, 1);
				color = vec3(percentage);
			}
			""",
			fragment_shader: """
			in vec3 color;
			void main()
			{
				output_image = color;
			}
			"""
		};
	};

	futures::executor::block_on(future);
}

fn compute_sum(numbers : &[f32]) -> f32
{
	let mut output : f32 = 0f32;
	let future = async_gpu_coordinator!
	{
		dispatch!
		{
			dimensions: [1, 1, 1],
			arguments: bind!{ numbers: & numbers, number_count: numbers.len(), output : &mut output},
			compute_kernel: """
			void main()
			{
				float sum = 0;
				for (int i = 0; i < number_count; i++)
				{
					sum += numbers[i];
				}
			}
			"""
		}
	};

	futures::executor::block_on(future);

	return output;
}

fn main()
{
	let image = Image::new(128, 128);
	draw_triangle(&[[-1.0, 0.0], [0.0, 1.0], [1.0, 0.0]], &mut image);
	let result = compute_sum(&[0f32, 1f32, 2f32, 3f32]);
}