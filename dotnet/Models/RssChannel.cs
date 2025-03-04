using System.Xml.Serialization;

namespace BackloggdBot.Models;

public class RssChannel
{
    [XmlElement(ElementName = "title")]
    public required string Title;
    [XmlElement(ElementName = "description")]
    public required string Description;
    [XmlElement(ElementName = "link")]
    public required string Link;
    [XmlElement(ElementName = "item")]
    public List<RssItem>? Items;
}
