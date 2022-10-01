use anyhow::{anyhow, Result};
use std::env;
use tokio::time::{sleep, Duration, Instant};
use tokio_postgres::{Client, Config, NoTls};

#[tokio::main]
async fn main() -> Result<()> {
    let client = connect().await?;

    let start_at = Instant::now();

    match send_query(client).await {
        Ok(_) => {
            let duration = Instant::now().duration_since(start_at).as_secs_f32();
            println!("Completed in {duration} secs");
            Ok(())
        }
        Err(e) => {
            println!("Ended with an error({:?})", e);
            Err(e)
        }
    }
}

async fn connect() -> Result<Client> {
    let mut config = Config::new();
    config
        .host("postgresql")
        .user("user")
        .password("pass")
        .dbname("db")
        .keepalives(true)
        .keepalives_idle(Duration::from_secs(60))
        .keepalives_interval(Duration::from_secs(15))
        .keepalives_retries(5);

    let mut errors = vec![];
    for _ in 0..10 {
        let res = config.connect(NoTls).await;
        match res {
            Ok((client, connection)) => {
                tokio::spawn(async move {
                    if let Err(e) = connection.await {
                        eprintln!("connection error: {}", e);
                    }
                });
                eprintln!("Connected.");
                return Ok(client);
            }
            Err(e) => {
                errors.push(e);
            }
        }

        sleep(Duration::from_secs(1)).await;
    }

    for e in errors {
        eprintln!("{:?}", e);
    }
    Err(anyhow!("Failed to connect"))
}

async fn send_query(client: Client) -> Result<()> {
    let sql = create_sql();
    client.query(sql.as_str(), &[]).await?;

    Ok(())
}

fn create_sql() -> String {
    let row_count = get_row_count();
    eprintln!("ROW_COUNT={row_count}");
    format!(
        r#"
        select
            t.n::integer           as c1,
            t.n::float             as c2,
            to_char(t.n, '999999') as c3
        from generate_series(1, {row_count}) as t(n)
        "#
    )
}

fn get_row_count() -> usize {
    let result = env::var("ROW_COUNT_LOG10")
        .map_err(|_| anyhow!("Env ROW_COUNT_LOG10 did not found."))
        .and_then(|val| {
            val.parse()
                .map_err(|_| anyhow!("Failed to parse ROW_COUNT_LOG10"))
        });

    match result {
        Ok(log10) => 10usize.pow(log10),
        Err(e) => {
            eprintln!("{:?}: Use default.", e);
            10usize.pow(4)
        }
    }
}
