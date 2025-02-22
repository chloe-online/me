use chrono::offset::Utc;
use chrono::{DateTime, Local};
use filetime::FileTime;
use html::content::{Footer, Header, Main};
use html::inline_text::Anchor;
use html::metadata::Head;
use html::root::{Body, Html};
use html::text_content::Division;
use regex::Regex;

use crate::index::SiteIndex;
use crate::web_page_file::WebPageFile;
use crate::SITE_NAME;

fn replace_file_links(
    input: &str,
    index: &SiteIndex,
    generate_link: fn(&str, Option<&str>) -> Anchor,
) -> Result<String, String> {
    // regex to match {file_name, optional[pretty_name]}
    let re = Regex::new(r"\{([^,}]+)(?:,([^}]+))?\}").unwrap();
    let mut result = String::new();

    let mut last_pos = 0;
    for caps in re.captures_iter(input) {
        let range = caps.get(0).unwrap().range();
        result.push_str(&input[last_pos..range.start]);

        let file_name = &caps[1];
        if !index.pages.contains(&file_name.to_string()) {
            return Err(format!("Found bad link '{}'", file_name));
        }

        let pretty_name = caps.get(2).map(|m| m.as_str());
        result.push_str(&generate_link(file_name, pretty_name).to_string());
        last_pos = range.end;
    }

    result.push_str(&input[last_pos..]);

    Ok(result)
}

fn generate_link(file_name: &str, pretty_name: Option<&str>) -> Anchor {
    let mut link = Anchor::builder();
    let file_link = format!("{}.html", file_name.to_string());
    match pretty_name {
        Some(name) => link.href(file_link).text(name.to_string()),
        None => link.href(file_link).text(file_name.to_string()),
    };

    link.build()
}

/// Represents a page on the website
#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct WebPage {
    pub name: String,
    pub content: String,
    pub date_edited: DateTime<Utc>,
}

impl WebPage {
    /// Constructs a new WebPage from an .htm source
    pub fn from_web_page_file(
        mut page_file: WebPageFile,
        index: &SiteIndex,
    ) -> Result<WebPage, String> {
        let contents = match page_file.get_page_contents() {
            Ok(val) => val,
            Err(e) => panic!("While reading file {},  {}", page_file.get_file_name(), e),
        };

        let name = String::from(page_file.file_path.file_stem().unwrap().to_str().unwrap());

        let modified_time = FileTime::from_last_modification_time(&page_file.metadata);
        let date_edited =
            DateTime::from_timestamp(modified_time.seconds(), modified_time.nanoseconds()).unwrap();

        match replace_file_links(&contents, &index, generate_link) {
            Ok(linked_content) => Ok(WebPage {
                name,
                content: linked_content,
                date_edited,
            }),
            Err(e) => Err(format!("While trying to build page '{}'...\n {}", name, e)),
        }
    }

    /// Converts a String into a WebPage
    pub fn from_string(name: String, content: String) -> WebPage {
        let date_edited: DateTime<Utc> = Local::now().into();
        WebPage {
            name,
            content,
            date_edited,
        }
    }

    pub fn get_formatted_time(&self) -> Result<String, Box<dyn std::error::Error>> {
        Ok(self
            .date_edited
            .with_timezone(&Local)
            .format("%I:%M%p, %b %e, %Y")
            .to_string()
            .to_lowercase())
    }

    /// Collects its content into the built version of the WebPage
    /// containing html header, navigation, and other page features.
    pub fn build(&self) -> String {
        let body = Body::builder()
            .push(self.get_header())
            .push(self.get_main())
            .push(self.get_footer())
            .build();

        let html = Html::builder()
            .lang("en")
            .push(self.get_head())
            .push(body)
            .build();

        html.to_string()
    }

    fn get_header(&self) -> Header {
        let mut header = Header::builder();
        let div = Division::builder()
            .class("special")
            .heading_1(|h1| {
                h1.anchor(|a| a.class("logo").href("index.html").text("*"))
                    .text(SITE_NAME)
            })
            .build();
        header.push(div);
        // add back to home nav for all non-home pages.
        if self.name != "home" {
            let back_to_home = Division::builder()
                .class("mini-indent")
                .anchor(|a| a.href("home.html").text("back to home"))
                .build();
            header.push(back_to_home);
        }

        header.build()
    }

    fn get_head(&self) -> Head {
        let head = Head::builder()
            .title(|title| title.text(format!("{} - {}", SITE_NAME, self.name)))
            .meta(|meta| {
                meta.name("description")
                    .content(format!("welcome to {}!!", SITE_NAME))
            })
            .link(|link| {
                link.rel("apple-touch-icon")
                    .sizes("180x180")
                    .href("../icons/apple-touch-icon.png")
            })
            .link(|link| link.rel("manifest").href("../site.manifest"))
            .meta(|meta| {
                meta.name("viewport")
                    .content("width=device-width, initial-scale=1.0")
            })
            .link(|link| link.href("../styles/style.css").rel("stylesheet"))
            .build();

        head
    }

    fn get_main(&self) -> Main {
        let main = Main::builder()
            .division(|inner| {
                inner
                    .class("inner")
                    .division(|indent| indent.class("indent").text(self.content.clone()))
            })
            .build();

        main
    }

    fn get_footer(&self) -> Footer {
        let gh_link = "https://github.com/andii-online/me";
        let last_edited_msg = format!("edited on {}", self.get_formatted_time().unwrap());
        let footer = Footer::builder()
            .division(|div| {
                div.class("left")
                    .paragraph(|p| p.anchor(|a| a.text("*website src").href(gh_link)))
            })
            .division(|div| div.class("right").paragraph(|p| p.text(last_edited_msg)))
            .build();

        footer
    }
}
