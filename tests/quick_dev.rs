#![allow(unused_imports)]
use anyhow::Result;

#[tokio::test]
async fn quick_dev() -> Result<()> {
    let hc = httpc_test::new_client("http://localhost:3000")?;
    //let res = hc.get("/hello").await?;

    hc.do_get("/hello").await?.print().await?;
    hc.do_get("/hello?name=Mike").await?.print().await?;
    hc.do_get("/hello2/Mike").await?.print().await?;

    //Actual endpoints
    hc.do_get("/login").await?.print().await?;
    Ok(())
}
