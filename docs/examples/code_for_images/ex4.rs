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

    cpu(x1);
    gpu(y1);

    cpu(x2);
    gpu(y2);

    fence(x1);
    fence(x2);
    fence(y1);
    fence(y2);

    cpu(xr);
    cpu(yr);
}