using System.Xml.Serialization;

namespace BackloggdBot.Models;

public class RssImage
{
    [XmlElement(ElementName = "url")]
    public required string Url;
}
