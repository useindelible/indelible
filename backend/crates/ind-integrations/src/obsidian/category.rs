use ind_domain::ItemType;

pub fn category_for_item_type(item_type: ItemType) -> &'static str {
    match item_type {
        ItemType::Book | ItemType::Pdf => "books",
        ItemType::Tweet => "tweets",
        ItemType::Podcast => "podcasts",
        ItemType::Article | ItemType::Email | ItemType::Video => "articles",
    }
}
