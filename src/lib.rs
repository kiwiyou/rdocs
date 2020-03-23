use kuchiki::traits::TendrilSink;
use element::{Document, parse_document};

pub mod element;

#[derive(Default)]
pub struct DocsClient {
    client: reqwest::Client,
}

impl DocsClient {
    pub async fn get_document(&self, package_name: &str, path: &str) -> Option<Document> {
        let url = self.get_path_url(package_name, path).await?;
        let data = reqwest::get(&url).await.ok()?.text().await.ok()?;
        let dom = kuchiki::parse_html().one(data);
        parse_document(&dom)
    }

    async fn get_path_url(&self, package_name: &str, path: &str) -> Option<String> {
        let path_parts: Vec<&str> = path.splitn(2, "::").collect();
        let url = get_docs_rs_url(package_name, path_parts[0]);
        if path_parts.len() == 2 {
            self.find_module(&url, path_parts[1])
                .await
                .or(self.find_sub_item(&url, path_parts[1]).await)
        } else {
            Some(url + "/index.html")
        }
    }

    async fn find_sub_item(&self, url: &str, sub_path: &str) -> Option<String> {
        let index_url = url.to_owned() + "/all.html";
        let data = self
            .client
            .get(&index_url)
            .send()
            .await
            .ok()?
            .text()
            .await
            .ok()?;
        let index_page = kuchiki::parse_html().one(data.as_ref());
        let link = index_page
            .select(".docblock > li > a")
            .unwrap()
            .find(|a| a.text_contents() == sub_path)?;
        let attributes = link.as_node().as_element()?.attributes.borrow();
        Some(url.to_owned() + "/" + attributes.get("href")?)
    }

    async fn find_module(&self, url: &str, sub_path: &str) -> Option<String> {
        let mut check_url = url.to_owned();
        for module in sub_path.split("::") {
            check_url.push('/');
            check_url.push_str(module);
        }
        let data = self.client.head(&check_url).send().await.ok()?;
        if data.status().is_success() {
            Some(check_url)
        } else {
            None
        }
    }
}

fn get_docs_rs_url(package_name: &str, crate_name: &str) -> String {
    format!(
        "https://docs.rs/{package}/*/{crate}",
        package = package_name,
        crate = crate_name
    )
}