use tokio::fs;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Attempt 1: write to a directory
    let res = fs::write(".", "test").await;
    println!("Write to '.' result: {:?}", res);
    
    // Attempt 2: write to root C:
    let res = fs::write("C:\\", "test").await;
    println!("Write to 'C:\\' result: {:?}", res);
    
    // Attempt 3: write to somewhere needing elevation
    let res = fs::write("C:\\Windows\\System32\\test.txt", "test").await;
    println!("Write to 'System32' result: {:?}", res);

    Ok(())
}
