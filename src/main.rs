use failure::Fallible;
use headless_chrome::{protocol::page::ScreenshotFormat, Browser};

use std::fs;
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::Duration;

use headless_chrome::browser::tab::RequestInterceptionDecision;
use headless_chrome::protocol::network::methods::RequestPattern;

fn main() -> Fallible<()> {
    search_devio()?;
    check_response()?;
    intercept_request()
}

fn search_devio() -> Fallible<()> {
    // ブラウザとタブの初期化
    let browser = Browser::default()?;
    let tab = browser.wait_for_initial_tab()?;
    tab.set_default_timeout(std::time::Duration::from_secs(200));

    // Googleを開く
    tab.navigate_to("https://www.google.com/")?;
    tab.wait_until_navigated()?;
    let jpeg_data = tab.capture_screenshot(ScreenshotFormat::JPEG(Some(75)), None, true)?;
    fs::write("screenshot.jpg", &jpeg_data)?;

    // 検索テキストボックスへフォーカス
    tab.wait_for_element("input[name=q]")?.click()?;
    // テキストボックスへ入力
    tab.type_str("DevelopersIO")?;
    let jpeg_data = tab.capture_screenshot(ScreenshotFormat::JPEG(Some(75)), None, true)?;
    fs::write("screenshot1-2.jpg", &jpeg_data)?;

    // 「I'm feeling lucky」ボタンを押下
    tab.wait_for_element("input[name=btnI]")?.click()?;
    sleep(Duration::from_secs(5));
    let jpeg_data = tab.capture_screenshot(ScreenshotFormat::JPEG(Some(75)), None, true)?;
    fs::write("screenshot2.jpg", &jpeg_data)?;

    Ok(())
}

fn check_response() -> Fallible<()> {
    // ブラウザとタブの初期化
    let browser = Browser::default()?;
    let tab = browser.wait_for_initial_tab()?;
    tab.set_default_timeout(std::time::Duration::from_secs(200));

    // レスポンスのハンドリングを有効化
    let responses = Arc::new(Mutex::new(Vec::new()));
    let responses2 = responses.clone();
    tab.enable_response_handling(Box::new(move |response, fetch_body| {
        sleep(Duration::from_millis(500));
        let body = fetch_body().unwrap();
        responses2.lock().unwrap().push((response, body));
    }))?;

    // Googleを開く
    tab.navigate_to("https://www.google.com/")?;
    tab.wait_until_navigated()?;

    let final_responses: Vec<_> = responses.lock().unwrap().clone();
    println!("{:#?}", final_responses);

    Ok(())
}

fn intercept_request() -> Fallible<()> {
    // ブラウザとタブの初期化
    let browser = Browser::default()?;
    let tab = browser.wait_for_initial_tab()?;
    tab.set_default_timeout(std::time::Duration::from_secs(200));

    // インターセプトする対象のパターンを指定
    let patterns = vec![RequestPattern {
        url_pattern: Some("https://www.google.com/"),
        resource_type: None,
        interception_stage: Some("Request"),
    }];

    // インターセプトを有効化
    tab.enable_request_interception(
        &patterns,
        Box::new(|_transport, _session_id, intercepted| {
            if intercepted.request.url == "https://www.google.com/" {
                println!("intercept!");
                let body = "This request was intercepted!";
                let js_response = tiny_http::Response::new(
                    200.into(),
                    vec![tiny_http::Header::from_bytes(
                        &b"Content-Type"[..],
                        &b"application/javascript"[..],
                    )
                    .unwrap()],
                    body.as_bytes(),
                    Some(body.len()),
                    None,
                );

                let mut wrapped_writer = Vec::new();
                js_response
                    .raw_print(&mut wrapped_writer, (1, 2).into(), &[], false, None)
                    .unwrap();

                let base64_response = base64::encode(&wrapped_writer);

                RequestInterceptionDecision::Response(base64_response)
            } else {
                println!("continue!");
                RequestInterceptionDecision::Continue
            }
        }),
    )?;

    // Googleを開く
    tab.navigate_to("https://www.google.com/")?;
    tab.wait_until_navigated()?;

    sleep(Duration::from_secs(5));
    let jpeg_data = tab.capture_screenshot(ScreenshotFormat::JPEG(Some(75)), None, true)?;
    fs::write("intercepted_screenshot.jpg", &jpeg_data)?;

    Ok(())
}
