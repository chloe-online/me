use crate::web_page::WebPage;
use crate::web_page_file::WebPageFile;

use html::content::Heading2;
use html::inline_text::Anchor;
use html::text_content::{Division, ListItem, UnorderedList};


pub struct SiteIndex {
    pub page: WebPage,
    pub pages: Vec<String>,
}

/// Creates the special site index WebPage.
pub fn get_site_index(pages: &Vec<WebPageFile>) -> SiteIndex {
    let mut index = Division::builder();
    let title = Heading2::builder().text("Site Index").build();
    index.push(title);

    let mut ul = UnorderedList::builder();
    for web_page in pages {
        let link_path = web_page
            .file_path
            .with_extension("html")
            .file_name()
            .unwrap()
            .to_string_lossy()
            .into_owned();
        let path_content = web_page
            .file_path
            .file_stem()
            .unwrap()
            .to_string_lossy()
            .into_owned();
        let anchor = Anchor::builder().href(link_path).text(path_content).build();
        let li = ListItem::builder().push(anchor).build();
        ul.push(li);
    }
    index.push(ul.build());

    let page = WebPage::from_string(String::from("index"), index.build().to_string());
    let pages: Vec<String> = pages.into_iter().map(|page| page.file_path.file_stem().unwrap().to_string_lossy().into_owned()).collect();

    SiteIndex {
        page,
        pages,
    }
}
