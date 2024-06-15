
use reqwest::Client;
use scraper::{Html, Selector};
use anyhow:: Result;
use qrcode::{render::svg, QrCode};
//use std::{collections::HashMap, thread};
use std:: thread;
use std::sync::mpsc;
use tauri::Manager;
use tokio::runtime::Runtime;
use chrono;
use std::sync::{Mutex, Once};
//use serde_json::{Map,Value};

static QRCODE_UUID: Mutex<Option<String>> = Mutex::new(None);
const WECHAT_REQUEST_QRCODE_URL: &str = "https://open.weixin.qq.com/connect/qrconnect?appid=wx4411fdc32430ce58&scope=snsapi_login&redirect_uri=https%3A%2F%2Faqy.lgb360.com%2F%23%2Flogin&state=123456";

static INIT: Once = Once::new();
//const LOGIN_URL:&str="https://aqy.lgb360.com/aqy/wechat/accountLogin";

async fn parse_html(html: &str) -> Result<String, ()> {
    let document = Html::parse_document(html);
    let qrcode_img_selector = Selector::parse("img.web_qrcode_img").map_err(|_| ())?;

    let qrcode_img_element = document
        .select(&qrcode_img_selector)
        .next()
        .ok_or(())?;

    let qrcode_img_src = qrcode_img_element
        .value()
        .attr("src")
        .ok_or(())?;

    let qrcode_href = qrcode_img_src.trim_start_matches('/');

    let qrcode_uuid = qrcode_href
        .rsplit('/')
        .next()
        .ok_or(())?;

    INIT.call_once(|| {
        let mut qrcode_uuid_guard = QRCODE_UUID.lock().unwrap();
        *qrcode_uuid_guard = Some(qrcode_uuid.to_string());
    });
    // 构建新的二维码 URL
    let barcode_url = format!(
    "https://open.weixin.qq.com/connect/confirm?uuid={}",
    qrcode_uuid
    );

    // 生成新的二维码图像
    let code = QrCode::new(barcode_url.as_bytes()).unwrap();

    let image = code.render::<svg::Color>().build();
    let svg_data = image.to_string();
    

    Ok(svg_data)
    
}

pub async fn get_qrcode_url() -> Result<String, ()> {
    let client = Client::new();
    let res = client.get(WECHAT_REQUEST_QRCODE_URL).send().await.map_err(|_| ())?;
    let html = res.text().await.map_err(|_| ())?;
    //println!("{}", html);
    parse_html(&html).await
}









pub async fn  monitor_wechat_scan(app_handle: &tauri::AppHandle) {
    let (tx, rx) = mpsc::channel();
    let qrcode_uuid_guard = QRCODE_UUID.lock().unwrap();
    let qrcode_uuid = qrcode_uuid_guard.as_ref().unwrap().clone();
    let window = app_handle.get_window("main").unwrap();
    thread::spawn(move || {
        let mut scan_qrcode_success = false;
        let wexin_poll_confirm_request_uuid_url = format!("https://lp.open.weixin.qq.com/connect/l/qrconnect?uuid={}&_={}", qrcode_uuid, "{}");
        
        for _ in 0..100 {
            std::thread::sleep(std::time::Duration::from_secs(1));
            let current_milli_time = chrono::Utc::now().timestamp_millis().to_string();
            let rt = Runtime::new().unwrap();
            let res_text = rt.block_on(async {
                reqwest::get(&wexin_poll_confirm_request_uuid_url.replace("{}", &current_milli_time))
                    .await
                    .unwrap()
                    .text()
                    .await
                    .unwrap()
            });

            if !res_text.contains("window.wx_errcode=405;window.wx_code=") {
                continue;
            }
            
            scan_qrcode_success = true;
            let wx_code = res_text.split("=").last().unwrap();
            let wx_code = wx_code.replace("'", "").replace(";", "");
            println!("{}",wx_code);
            let cloned_wx_code = wx_code.clone();
            tx.send(wx_code).unwrap();

            let client = Client::new();
            let mut url = "https://aqy.lgb360.com/aqy/wechat/accountLogin".to_owned();
            let type_: String  ="3".to_string();
            /*let body = [("code", &cloned_wx_code), ("type", &type_)]
                .iter()
                .cloned()
                .collect::<HashMap<_, _>>();
            let request_builder = client
                .post(url)
                .json(&body);*/
            // 将参数拼接到 URL 中
            url.push_str(&format!("?code={}&type={}", cloned_wx_code, type_));

            let request_builder = client.post(url);
            // 设置 header
            let request_builder = request_builder
                .header("Host", "aqy.lgb360.com")
                .header("Origin", "https://aqy.lgb360.com")
                .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/113.0.0.0 Safari/537.36")
                .header("accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7")
                .header("Content-Type", "application/json")
                .header("Referer", "https://aqy.lgb360.com/")
                //.header("accept-encoding", "gzip, deflate, br")
                .header("accept-language", "zh-CN,zh;q=0.9,ja;q=0.8,en;q=0.7,ko;q=0.6")
                .header("Connection", "Keep-Alive")
                .header("sec-ch-ua", "\"Google Chrome\";v=\"113\", \"Chromium\";v=\"113\", \"Not-A.Brand\";v=\"24\"")
                .header("sec-ch-ua-mobile", "?0")
                .header("sec-ch-ua-platform", "Windows")
                .header("sec-fetch-site", "same-origin")
                .header("sec-fetch-mode", "cors")
                .header("sec-fetch-dest", "empty");

            // 设置 cookie
            //let cookie_string = "cookie1=value1; cookie2=value2";
            //let request_builder = request_builder.header("Cookie", cookie_string);

            let request = request_builder.build().unwrap();

            let response = rt.block_on(async {client
                .execute(request)
                .await
            });
            //let json_data: Map<String, Value> = response.json().unwrap();

            // 处理 JSON 数据
            println!("JSON 数据: {:?}", response);
            match response {
                Ok(response) => {
                    
                    let response_bytes = rt.block_on(async {response.bytes().await.unwrap()});
                    println!("解压的JSON 数据: {:?}", std::str::from_utf8(response_bytes.as_ref()));
                    
                }
                Err(err) => {
                    println!("请求失败: {:?}", err);
                    
                }
            };

            window.emit("scan-result", Some(cloned_wx_code)).unwrap();
            break;
        }

        if !scan_qrcode_success {
            tx.send("scan_timeout".to_string()).unwrap();
            window.emit("scan-result", Some("scan_timeout")).unwrap();
        }
    });
   



    let window = app_handle.get_window("main").unwrap();
    //println!("获取window{:?}",&window);
    window.once("scan-result", move |_| {
        println!("收到结果");
        match rx.recv() {
            //result
            Ok(_) => {
               println!("收到结果");
                /*  if result == "scan_timeout" {
                    println!("微信扫码超时");
                } else {
                    println!("微信扫码成功，后端获取到 wx_code: {}", result);
                }*/
            }
            Err(err) => {
                println!("Error receiving scan result: {}", err);
            }
        }
    });
}
