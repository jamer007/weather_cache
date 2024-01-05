use scraper::{Html, Selector};

async fn get_html_forecast_page(url: &str) -> String {
    let ua = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36 OPR/106.0.0.0";
    let req = surf::get(url).header("User-Agent", ua);
    let client = surf::client().with(surf::middleware::Redirect::new(5));
    client.send(req).await.unwrap().body_string().await.unwrap()
}

pub async fn get_forecast_temps(location_id: String) -> Vec<f32> {
    let url = format!("https://www.accuweather.com/en/ca/city/h0h/weather-forecast/{location_id}");
    let body = get_html_forecast_page(url.leak()).await;
    let document = Html::parse_document(&body);
    find_temps_in_html(document)
}

fn find_temps_in_html(document: Html) -> Vec<f32> {
    let temp_selector = Selector::parse("div.temp").unwrap();
    let temp_container = document.select(&temp_selector).next().unwrap().inner_html();
    let real_temp_selector = Selector::parse("div.real-feel").unwrap();
    let real_temp_container = document.select(&real_temp_selector).next().unwrap().inner_html();
    let temp = temp_number(temp_container);
    let real_temp = real_temp_number(real_temp_container);
    vec![temp, real_temp]
}

fn temp_number(temp: String) -> f32 {
    let temp = temp.replace("<span", "");
    let temp = temp.split(" ").collect::<Vec<_>>()[0];
    let temp = temp.replace("°", "");
    temp.parse::<f32>().unwrap()
}
fn real_temp_number(temp: String) -> f32 {
    let temp = temp.split("RealFeel®").collect::<Vec<_>>();
    let temp = temp[1].replace("°", "");
    let temp = temp.replace("\t", "");
    let temp = temp.replace("\n", "");
    temp.parse::<f32>().unwrap()
}