use failure::Fallible;
// use std::error::Error;
// use failure::Fallible;

use headless_chrome::Browser;

fn main() -> Fallible<()> {
    let browser = Browser::default()?;

    let tab = browser.wait_for_initial_tab()?;

    tab.navigate_to("https://www.wikipedia.org/")?;
    tab.wait_for_element("input#searchInput")?.click()?;

    tab.type_str("WebKit")?;
    tab.press_key("Enter")?;
    tab.wait_for_element("#firstHeading")?;

    println!("{}", tab.get_url());

    assert_eq!(true, tab.get_url().ends_with("WebKit"));

    Ok(())
}
