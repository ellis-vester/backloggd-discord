using System.Xml;
using System.Xml.Serialization;
using BackloggdBot.Models;
using Serilog;

namespace BackloggdBot.Scraper;

public class Scraper
{
    public async Task<RssFeed> GetRssFeedContent(string url)
    {
        var client = new HttpClient();

        var response = await client.GetAsync(url);

        if (response.IsSuccessStatusCode)
        {
            if (response.Content == null)
                throw new Exception($"Null response from RSS url {url}");
            else
            {
                var serializer = new XmlSerializer(typeof(RssFeed));

                Log.Logger.Information(await response.Content.ReadAsStringAsync());

                using (var stringReader = new StringReader(await response.Content.ReadAsStringAsync()))
                using (var xmlTextReader = new XmlTextReader(stringReader))
                {
                    return (RssFeed)serializer.Deserialize(xmlTextReader);
                }
            }
        }
        else
        {
            throw new Exception($"No content at RSS url {url}");
        }
    }
}
