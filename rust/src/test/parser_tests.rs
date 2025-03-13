#[test]
fn parse_rss_xml_returns_rss_channel() {
    let rss_content = r#"<rss version="2.0">
            <channel>
                <title>Backloggd</title>
                <description>Backloggd</description>
                <link>https://www.backloggd.com/</link>
                <item>
                    <title>Item1</title>
                    <link>https://www.backloggd.com/</link>
                    <pubDate>Sat, 04 May 2024 01:05:21 +0000</pubDate>
                    <description>Description1</description>
                    <guid isPermaLink="false">backloggd-review-0000001</guid>
                    <backloggd:user_rating>1</backloggd:user_rating>
                    <backloggd:reviewer>username1</backloggd:reviewer>
                    <image>
                        <url>https://images.igdb.com/igdb/image/1.jpg</url>
                    </image>
                </item>
                <item>
                    <title>Item2</title>
                    <link>https://www.backloggd.com/</link>
                    <pubDate>Sat, 04 May 2024 01:05:21 +0000</pubDate>
                    <description>Description2</description>
                    <guid isPermaLink="false">backloggd-review-0000002</guid>
                    <backloggd:user_rating>2</backloggd:user_rating>
                    <backloggd:reviewer>username2</backloggd:reviewer>
                    <image>
                        <url>https://images.igdb.com/igdb/image/2.jpg</url>
                    </image>
                </item>
            </channel>
        </rss>"#;

    let actual = crate::core::parser::parse_rss_xml(&rss_content);

    match actual {
        Ok(value) => {
            assert_eq!(value.channel.title, "Backloggd");
            assert_eq!(value.channel.link, "https://www.backloggd.com/");
            assert_eq!(value.channel.description, "Backloggd");

            assert_eq!(value.channel.item[0].title, "Item1");
            assert_eq!(value.channel.item[0].user_rating, 1);
            assert_eq!(
                value.channel.item[0].image.url,
                "https://images.igdb.com/igdb/image/1.jpg"
            );

            assert_eq!(value.channel.item[1].title, "Item2");
            assert_eq!(value.channel.item[1].user_rating, 2);
            assert_eq!(
                value.channel.item[1].image.url,
                "https://images.igdb.com/igdb/image/2.jpg"
            );
        }
        Err(error) => {
            panic!("Error during parse of XML {}", error);
        }
    }
}
