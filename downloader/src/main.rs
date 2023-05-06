use std::path::PathBuf;

use anyhow::bail;
use regex::Regex;
use tracing::{debug, info, trace};

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

    let (tx, mut rx) = tokio::sync::mpsc::channel(100);

    let mut tasks = Vec::new();
    for i in &rfcs {
        let client = client.clone();
        let url = i.url();
        let path = i.path();
        let tx = tx.clone();

        tasks.push(tokio::spawn(async move {
            let txt = client.get(&url).send().await?.text().await?;
            trace!(?path, "downloaded");
            tx.send((path, txt)).await?;

            Result::<_, anyhow::Error>::Ok(())
        }));
    }

    info!("started all downloads");

    tokio::task::spawn_blocking(move || -> anyhow::Result<()> {
        info!("starting writer");
        while let Some((path, txt)) = rx.blocking_recv() {
            trace!(?path, "writing");
            std::fs::write(path, txt)?;
        }
        info!("done writing");
        Ok(())
    });

    for t in tasks {
        t.await??;
    }

    Ok(())
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
