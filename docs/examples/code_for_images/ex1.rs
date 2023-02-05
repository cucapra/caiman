fn value() {
    let x : i32 = 5;            
    let y : i32 = 7;
    let z : i32 = x + y;
}

fn schedule() {
    cpu(x);
    gpu(y);    
    fence(x);
    fence(y);
    cpu(z);
}