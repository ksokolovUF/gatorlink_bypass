use anyhow::{Context, Result, anyhow, bail};
use std::{fs, process::Command, time::Duration};
use thirtyfour::extensions::query::ElementQueryable;
use thirtyfour::{By, DesiredCapabilities, Key, WebDriver};
use tokio::time::{sleep, timeout};

const PORT: u16 = 4445;

struct ChildGuard(std::process::Child);
impl Drop for ChildGuard {
    // destructor kills the process
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

pub fn copy_wayland(text: &str) {
    std::process::Command::new("wl-copy")
        .arg("--") // stop option parsing (so text starting with '-' won’t be treated as a flag)
        .arg(text) // wl-copy accepts text as args
        .status()
        .unwrap();
}

fn pop_last_line(path: &str) -> Result<String> {
    let s = fs::read_to_string(path)?;
    let mut v: Vec<&str> = s.lines().collect();
    let last = v
        .pop()
        .map(str::to_string)
        .ok_or_else(|| anyhow!("empty"))?;
    fs::write(path, v.join("\n") + "\n")?;
    copy_wayland(&last);
    Ok(last)
}

async fn renew_codes() -> Result<()> {
    // read secrets
    let username = fs::read_to_string(".secrets/username.txt")
        .context("could not read .secrets/username.txt")?
        .trim()
        .to_string();
    let password = fs::read_to_string(".secrets/password.txt")
        .context("could not read .secrets/password.txt")?
        .trim()
        .to_string();

    // run geckodriver
    let geckodriver = std::env::args()
        .nth(1)
        .context("missing geckodriver path argument (pass it as argv[1])")?;

    let _gecko = ChildGuard(
        Command::new(&geckodriver)
            .arg("--port")
            .arg(PORT.to_string())
            .spawn()
            .with_context(|| format!("failed to start geckodriver: {geckodriver}"))?,
    );

    // setup webdriver
    let caps = DesiredCapabilities::firefox();
    //let mut caps = DesiredCapabilities::firefox();
    //caps.set_headless()?;

    // retry connect a few times to avoid startup race
    let server = format!("http://localhost:{PORT}");
    let mut driver = None;
    for _ in 0..10 {
        match WebDriver::new(&server, caps.clone()).await {
            Ok(d) => {
                driver = Some(d);
                break;
            }
            Err(_) => sleep(Duration::from_millis(150)).await,
        }
    }
    let driver = driver.context("could not connect to geckodriver")?;
    driver
        .set_implicit_wait_timeout(Duration::from_secs(10))
        .await?;

    let result: Result<()> = async {
        driver.goto("https://account.it.ufl.edu/").await?;
        let button = driver
            .query(By::Css("a[href='/glam/passcodes']"))
            .first()
            .await?;
        button.scroll_into_view().await?;
        button.click().await?;
        let u_form = driver.query(By::Id("username")).first().await?;
        let p_form = driver.query(By::Id("password")).first().await?;
        u_form.clear().await?;
        u_form.send_keys(&username).await?;
        p_form.clear().await?;
        p_form.send_keys(&password).await?;
        p_form.send_keys(Key::Enter).await?;
        driver
            .find(By::XPath("//button[normalize-space(.)='Other options']"))
            .await?
            .click()
            .await?;
        driver
            .query(By::XPath(
                "//div[contains(concat(' ', normalize-space(@class), ' '), ' method-label ') \
          and normalize-space(.)='Bypass code']",
            ))
            .first()
            .await?
            .click()
            .await?;
        let code_form = driver.query(By::Id("passcode-input")).first().await?;
        let code = pop_last_line(".secrets/codes.txt")?;

        code_form.send_keys(code).await?;
        code_form.send_keys(Key::Enter).await?;

        driver
            .find(By::Id("dont-trust-browser-button"))
            .await?
            .click()
            .await?;

        driver
            .query(By::Id("gen_bypass_codes_btn"))
            .wait(Duration::from_secs(10), Duration::from_millis(200))
            .first()
            .await?
            .click()
            .await?;
        // wait until the codes div has non-empty text
        let text = timeout(Duration::from_secs(30), async {
            loop {
                let t = driver
                    .find(By::Id("generated_bypass_codes"))
                    .await?
                    .text()
                    .await?;

                if !t.trim().is_empty() {
                    return Ok::<_, anyhow::Error>(t);
                }

                sleep(Duration::from_millis(150)).await;
            }
        })
        .await
        .map_err(|_| anyhow!("timed out waiting for bypass codes"))??;

        let new_codes: String = text
            .lines()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join("\n");
        std::fs::write(".secrets/codes.txt", new_codes)?;
        Ok(())
    }
    .await;

    // try to quit even if something failed
    let _ = driver.quit().await;
    result
}

#[tokio::main]
async fn main() -> Result<()> {
    let codes = fs::read_to_string(".secrets/codes.txt")
        .context("could not read .secrets/codes.txt")?
        .trim()
        .to_string();
    let code_count = codes.lines().filter(|l| !l.trim().is_empty()).count();
    if codes.is_empty() {
        bail!("codes.txt is empty");
    } else if code_count == 1 {
        let _ = renew_codes().await?;
    } else {
        pop_last_line(".secrets/codes.txt")?;
    }
    Ok(())
}
