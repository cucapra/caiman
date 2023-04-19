struct Input_0 { field_0 : f32 };
struct Input_1 { field_0 : f32 };
struct Output_0 { field_0 : f32 };

@group(0) @binding(0)
var<storage, read> a : Input_0;
@group(0) @binding(1)
var<storage, read> b : Input_1;
@group(0) @binding(2)
var<storage, write> c : Output_0;

@compute @workgroup_size(1)
fn main()
{
    c.field_0 = a.field_0 * b.field_0;
}