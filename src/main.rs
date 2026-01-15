use anyhow::{Context, Result, bail};
use std::{fs, process::Command, time::Duration};
use thirtyfour::extensions::query::ElementQueryable;
use thirtyfour::{By, DesiredCapabilities, Key, WebDriver};
use tokio::time::sleep;

const PORT: u16 = 4445;

struct ChildGuard(std::process::Child);
impl Drop for ChildGuard {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // read secrets
    let codes = fs::read_to_string(".secrets/codes.txt")
        .context("could not read .secrets/codes.txt")?
        .trim()
        .to_string();
    if codes.is_empty() {
        bail!("codes.txt is empty");
    }
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
        .unwrap_or_else(|| "geckodriver".into());

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
        .set_implicit_wait_timeout(Duration::from_secs(5))
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
        //sleep(Duration::from_millis(450)).await;
        code_form.send_keys("12345").await?;

        /*
        driver
            .find(By::Id("gen_bypass_codes_btn"))
            .await?
            .click()
            .await?;
        */
        Ok(())
    }
    .await;

    // try to quit even if something failed
    let _ = driver.quit().await;
    result
}
