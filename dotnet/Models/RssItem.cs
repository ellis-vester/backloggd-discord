using System.Xml.Serialization;

namespace BackloggdBot.Models;

public class RssItem
{
    [XmlElement(ElementName = "title")]
    public required string Title;
    [XmlElement(ElementName = "link")]
    public required string Link;
    [XmlElement(ElementName = "pubDate")]
    public required string PublicationDate;
    [XmlElement(ElementName = "description")]
    public required string Description;
    [XmlElement(ElementName = "guid")]
    public required string Guid;
    [XmlElement(ElementName = "backloggd:user_rating")]
    public required int UserRating;
    [XmlElement(ElementName = "backloggd:reviewer")]
    public required string Reviewer;
    [XmlElement(ElementName = "image")]
    public required RssImage Image;
}
