use std::path::PathBuf;

use anyhow::{bail, Context};
use regex::Regex;
use tracing::{debug, info, trace, warn};

const RFC_INDEX_URL: &str = "https://www.rfc-editor.org/rfc-index.txt";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    do_main().await
}

async fn do_main() -> anyhow::Result<()> {
    let client = reqwest::Client::builder()
        .user_agent("github.com/aDotInTheVoid/except")
        .build()?;

    let index = client.get(RFC_INDEX_URL).send().await?.text().await?;
    // include_str!("../rfc-index.txt");

    let Some((_, index)) = index
        .rsplit_once(
            "~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~",
        ) else {
            bail!("RFC index format changed")
        };

    let line_start_regex = Regex::new(r"^[0-9]{4}")?;
    let format_regex = Regex::new(r"\(Format: .*TXT.*\)")?;

    let rfcs = index
        .split("\n\n")
        .filter(|l| line_start_regex.is_match(l))
        .filter(|l| !l.contains("Not Issued."))
        .map(|s| s.replace("\n     ", " "))
        .map(|s| Rfc {
            num: s[0..4].parse().unwrap(),
            has_txt: format_regex.is_match(&s),
        })
        .filter(|i| i.has_txt)
        .filter(|i| !i.path().exists()) // This tecnicly blocks on IO, but eh
        .collect::<Vec<_>>();

    info!("found {} RFCs that need downloading", rfcs.len());

    let (tx, mut rx) = tokio::sync::mpsc::channel::<(PathBuf, String)>(100);

    let mut tasks = Vec::new();
    info!("starting downloads");
    for i in &rfcs {
        let client = client.clone();
        let url = i.url();
        let path = i.path();
        let tx = tx.clone();

        tasks.push(tokio::spawn(async move {
            let txt = get_url(&client, &url).await?;
            trace!(?path, "downloaded");
            tx.send((path, txt))
                .await
                .context("Couldn't send downloaded rfc")?;

            Result::<_, anyhow::Error>::Ok(())
        }));
    }
    info!("started all downloads");

    let jh = tokio::task::spawn_blocking(move || -> anyhow::Result<()> {
        info!("starting writer");
        while let Some((path, txt)) = rx.blocking_recv() {
            trace!(?path, "writing");
            std::fs::create_dir_all(path.parent().unwrap())?;
            std::fs::write(&path, txt)?;
            trace!(?path, "wrote");
        }
        info!("done writing");
        rx.close();
        Ok(())
    });

    info!("waiting for {} downloads to finish", tasks.len());
    let mut n_done = 0;
    for t in tasks {
        match t.await? {
            Ok(_) => {}
            Err(e) => {
                warn!(%e, "error downloading");
            }
        };
        n_done += 1;
        if n_done % 200 == 0 {
            debug!(%n_done, "Making progress");
        }
    }
    info!("all downloads finished");

    drop(tx);

    jh.await??;

    Ok(())
}

async fn get_url(client: &reqwest::Client, url: &str) -> anyhow::Result<String> {
    // async fn get_inner(client: &reqwest::Client, url: &str) -> Result<String, reqwest::Error> {
    //     let txt =
    //     Ok(txt)
    // }

    Ok(
        backoff::future::retry(backoff::ExponentialBackoff::default(), || async move {
            Ok(client
                .get(url)
                .send()
                .await?
                .error_for_status()?
                .text()
                .await?)
        })
        .await?,
    )
}

#[derive(Debug)]
struct Rfc {
    num: u32,
    has_txt: bool,
}

impl Rfc {
    fn url(&self) -> String {
        format!("https://www.rfc-editor.org/rfc/rfc{}.txt", self.num)
    }

    fn path(&self) -> PathBuf {
        let n = format!("{:04}", self.num);
        let dir = &n[0..2];

        format!("rfcs/{dir}/rfc{n}.txt").into()
    }
}
