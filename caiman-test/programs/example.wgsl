struct InputOutput_0 { field_0 : i32; };
[[group(0), binding(0)]]
var<storage, read_write> input_output_0 : InputOutput_0;
[[stage(compute), workgroup_size(1, 1, 1)]]
fn main() // must be called main for now!
{
    input_output_0.field_0 = input_output_0.field_0 + 1;
}