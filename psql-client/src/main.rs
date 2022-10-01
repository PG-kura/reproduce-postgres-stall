use anyhow::{anyhow, Result};
use std::env;
use std::process::{ExitStatus, Stdio};
use tokio::io::AsyncWriteExt;
use tokio::process::{Child, Command};
use tokio::time::{sleep, Duration, Instant};

#[tokio::main]
async fn main() -> Result<()> {
    let mut errors = vec![];

    for _ in 0..10 {
        sleep(Duration::from_secs(1)).await;

        let child = match connect().await {
            Ok(child) => child,
            Err(e) => {
                errors.push(e);
                continue;
            }
        };

        let start_at = Instant::now();

        let res = match send_query(child).await {
            Ok(status) => match status.code() {
                Some(2) => {
                    eprintln!("Failed to connect, psql exited immediately. Retrying");
                    continue;
                }
                code => {
                    let duration = Instant::now().duration_since(start_at).as_secs_f32();
                    println!("Completed in {duration} secs with status code {:?}", code);
                    Ok(())
                }
            },
            Err(e) => Err(e),
        };

        return res;
    }

    for e in errors {
        eprintln!("{:?}", e);
    }

    Err(anyhow!("Failed to send query"))
}

async fn connect() -> Result<Child> {
    let child = Command::new("psql")
        .arg("-h")
        .arg("postgresql")
        .arg("-U")
        .arg("user")
        .arg("-a")
        .arg("db")
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();
    match child {
        Ok(mut child) => {
            if child.stdin.is_some() {
                eprintln!("Connected.");
                Ok(child)
            } else {
                child.kill().await.ok();
                Err(anyhow!("Child did not have a handle to stdin"))
            }
        }
        Err(e) => Err(anyhow!(e)),
    }
}

async fn send_query(mut child: Child) -> Result<ExitStatus> {
    let mut stdin = child.stdin.take().unwrap();

    let sql = create_sql();
    stdin
        .write(sql.as_bytes())
        .await
        .expect("could not write to stdin");

    drop(stdin);

    child.wait().await.map_err(|e| anyhow!(e))
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
        .map_err(|_| anyhow!("Env ROW_COUNT did not found."))
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
