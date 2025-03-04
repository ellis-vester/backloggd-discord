using System.Xml.Serialization;

namespace BackloggdBot.Models;

[XmlRoot("rss")]
public class RssFeed
{
    [XmlElement(ElementName = "channel")]
    public required RssChannel Channel;
}
