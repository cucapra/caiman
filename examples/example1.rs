struct Image
{

}

fn draw_triangle(vertices : &[[f32; 3]; 3], output_image : &mut Image)
{
	synchronous_gpu_coordinator! {
		draw! {
			element_count: 3,
			arguments: bind!{ vertices : & vertices, output_image : &mut output_image },
			vertex_shader: """
			void main()
			{
				gl_Position = vec4(vertices[gl_VertexIndex], 0, 1);
			}
			""",
			fragment_shader: """
			void main()
			{
				output_image = vec4(1, 0, 0, 1);
			}
			"""
		};
	};
}

fn compute_sum(numbers : &[f32]) -> f32
{
	let mut output : f32 = 0f32;
	synchronous_gpu_coordinator! {
		dispatch! {
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

	return output;
}

fn main()
{
	let image = Image::new(128, 128);
	draw_triangle(&[[-1.0, 0.0], [0.0, 1.0], [1.0, 0.0]], &mut image);
	let result = compute_sum(&[0f32, 1f32, 2f32, 3f32]);
}