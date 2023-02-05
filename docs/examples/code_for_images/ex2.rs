fn value() {

    let x : i32 = 5;            
    let y : i32 = 7;
    let z : i32 = x + y;
}

fn schedule() {
    cpu(x);
    cpu(y);    
    fence(x);
    fence(y);
    gpu(z);
}