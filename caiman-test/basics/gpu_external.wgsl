struct Output {field_0 : i32;};
fn trivial(a : i32) -> Output
{
    var output : Output;
    output.field_0 = a + 1;
    return output;
}

struct Input_0 { field_0 : i32; };
[[group(0), binding(0)]] var<storage, read> input_0 : Input_0;
struct Output_0 { field_0 : i32; };
[[group(0), binding(1)]] var<storage, read_write> output_0 : Output_0;
[[stage(compute), workgroup_size(1, 1, 1)]] fn main()
{
    let output = trivial(input_0.field_0);
    output_0.field_0 = output.field_0;
}