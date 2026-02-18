use reqwest::Client;
use serde::{Deserialize, Serialize};
use chrono::{NaiveDate, Utc};
use regex::Regex;
use scraper::{Html, Selector};


#[derive(Debug, Serialize, Deserialize)]
struct RemoteTokenResponse {
    token: String,
    expires_at: chrono::DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
struct LoginRequest {
    email: String,
    password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CreateEventRequest {
    title: String,
    description: Option<String>,
    event_type: Option<String>,
    dance_styles: Vec<String>,
    start_date: NaiveDate,
    end_date: Option<NaiveDate>,
    venue_name: Option<String>,
    address: Option<String>,
    city: String,
    state: String,
    zip_code: Option<String>,
    latitude: Option<f64>,
    longitude: Option<f64>,
    ticket_price: Option<f64>,
    max_attendees: Option<u32>,
    image_url: Option<String>,
    website_url: Option<String>,
    contact_email: Option<String>,
    contact_phone: Option<String>,
}

struct EventScraper {
    client: Client,
    api_base: String,
    token: Option<String>,
}

impl EventScraper {
    fn new(api_base: String) -> Self {
        Self {
            client: Client::new(),
            api_base,
            token: None,
        }
    }

    async fn get_remote_token(&mut self) -> anyhow::Result<()> {
        let login_data = LoginRequest {
            email: "admin@danceconnect.com".to_string(),
            password: "admin123".to_string(),
        };

        let response: RemoteTokenResponse = self
            .client
            .post(&format!("{}/api/login_remotely", self.api_base))
            .json(&login_data)
            .send()
            .await?
            .json()
            .await?;

        let token = response.token.clone();
        self.token = Some(response.token);
        println!("Got remote token: {}", token);
        Ok(())
    }

    async fn submit_event(&self, event: CreateEventRequest) -> anyhow::Result<()> {
        if self.token.is_none() {
            return Err(anyhow::anyhow!("No token available"));
        }

        let response = self
            .client
            .post(&format!("{}/api/register_event_remotely", self.api_base))
            .header("Authorization", format!("Bearer {}", self.token.as_ref().unwrap()))
            .json(&event)
            .send()
            .await?;

        if response.status().is_success() {
            println!("Successfully submitted event: {}", event.title);
        } else {
            println!("Failed to submit event: {}", response.status());
        }

        Ok(())
    }

    fn parse_telegram_event(&self, text: &str) -> Option<CreateEventRequest> {
        let dance_keywords = ["dance", "salsa", "bachata", "tango", "swing", "ballroom", "latin", "hip-hop", "contemporary"];
        if !dance_keywords.iter().any(|&keyword| text.to_lowercase().contains(keyword)) {
            return None;
        }

        let lines: Vec<&str> = text.lines().collect();
        let title = lines.first()?.trim().to_string();
        
        // Extract date using regex
        let date_regex = Regex::new(r"(\d{1,2}[/-]\d{1,2}[/-]\d{2,4}|\d{4}-\d{2}-\d{2})").ok()?;
        let start_date = if let Some(date_match) = date_regex.find(text) {
            chrono::NaiveDate::parse_from_str(date_match.as_str(), "%Y-%m-%d")
                .or_else(|_| chrono::NaiveDate::parse_from_str(date_match.as_str(), "%m/%d/%Y"))
                .unwrap_or_else(|_| chrono::Utc::now().date_naive() + chrono::Duration::days(7))
        } else {
            chrono::Utc::now().date_naive() + chrono::Duration::days(7)
        };
        
        // Extract venue/location
        let venue_regex = Regex::new(r"(?i)at\s+([^\n]+)").ok()?;
        let venue_name = venue_regex.find(text)
            .map(|m| m.as_str().trim_start_matches("at ").trim().to_string())
            .or_else(|| Some("TBA".to_string()));
            
        // Detect dance styles
        let mut dance_styles = Vec::new();
        for &style in &dance_keywords {
            if text.to_lowercase().contains(style) {
                dance_styles.push(style.to_string());
            }
        }
        if dance_styles.is_empty() {
            dance_styles.push("social".to_string());
        }
        
        Some(CreateEventRequest {
            title,
            description: Some(text.to_string()),
            event_type: Some("social".to_string()),
            dance_styles,
            start_date,
            end_date: None,
            venue_name,
            address: Some("TBA".to_string()),
            city: "Unknown".to_string(),
            state: "Unknown".to_string(),
            zip_code: None,
            latitude: None,
            longitude: None,
            ticket_price: None,
            max_attendees: None,
            image_url: None,
            website_url: None,
            contact_email: None,
            contact_phone: None,
        })
    }

    async fn scan_telegram(&self) -> Vec<CreateEventRequest> {
        // Mock telegram scanning with more realistic messages
        let mock_messages = vec![
            "🕺 Salsa Night at Downtown Studio\nDate: 2024-12-28\nJoin us for an amazing night of salsa dancing!\nLocation: 123 Dance Street, NYC",
            "💃 Bachata Workshop this Saturday\nDecember 30, 2024\nLearn the basics of bachata with professional instructors\nat Dance Academy LA",
            "🎵 Latin Dance Social\nJanuary 3, 2025\nCome dance salsa, bachata, and merengue with us!\nVenue: Community Center Chicago",
            "🎭 Tango Milonga\n2025-01-10\nAuthentic Argentine Tango night\nat Tango Palace, Miami",
            "🕺 Hip-Hop Battle Championship\nJan 15, 2025\nBest dancers compete for prizes\nUrban Dance Studio, Atlanta"
        ];

        println!("Processing {} Telegram messages...", mock_messages.len());
        let events: Vec<CreateEventRequest> = mock_messages
            .iter()
            .filter_map(|msg| {
                let event = self.parse_telegram_event(msg);
                if event.is_some() {
                    println!("✓ Parsed Telegram event: {}", msg.lines().next().unwrap_or("Unknown"));
                }
                event
            })
            .collect();
            
        println!("Found {} events from Telegram", events.len());
        events
    }

    async fn scrape_dance_websites(&self) -> Vec<CreateEventRequest> {
        let mut events = Vec::new();
        
        // Mock scraping multiple dance websites
        let mock_html_events = vec![
            ("<h2>Salsa Fever Night</h2><p>Date: 2024-12-25</p><p>Location: Studio 54</p>", "New York", "NY"),
            ("<h3>Tango Workshop</h3><p>December 30, 2024</p><p>Venue: Dance Palace</p>", "Los Angeles", "CA"),
            ("<div class='event'>Swing Dance Social<br>Jan 5, 2025<br>Community Center</div>", "Chicago", "IL"),
        ];
        
        for (html_content, city, state) in mock_html_events {
            if let Some(event) = self.parse_html_event(html_content, city, state) {
                events.push(event);
            }
        }
        
        events
    }
    
    fn parse_html_event(&self, html: &str, city: &str, state: &str) -> Option<CreateEventRequest> {
        let document = Html::parse_fragment(html);
        
        // Extract title
        let title_selectors = ["h1", "h2", "h3", ".title", ".event-title"];
        let mut title = String::new();
        for selector_str in &title_selectors {
            if let Ok(selector) = Selector::parse(selector_str) {
                if let Some(element) = document.select(&selector).next() {
                    title = element.text().collect::<String>().trim().to_string();
                    break;
                }
            }
        }
        
        if title.is_empty() {
            title = "Web Scraped Dance Event".to_string();
        }
        
        // Extract date
        let text_content = document.root_element().text().collect::<String>();
        let date_regex = Regex::new(r"(\d{1,2}[/-]\d{1,2}[/-]\d{2,4}|\d{4}-\d{2}-\d{2}|[A-Za-z]+ \d{1,2}, \d{4})").ok()?;
        let start_date = if let Some(date_match) = date_regex.find(&text_content) {
            let date_str = date_match.as_str();
            chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
                .or_else(|_| chrono::NaiveDate::parse_from_str(date_str, "%B %d, %Y"))
                .or_else(|_| chrono::NaiveDate::parse_from_str(date_str, "%b %d, %Y"))
                .unwrap_or_else(|_| chrono::Utc::now().date_naive() + chrono::Duration::days(10))
        } else {
            chrono::Utc::now().date_naive() + chrono::Duration::days(10)
        };
        
        // Detect dance styles from title and content
        let dance_keywords = ["salsa", "bachata", "tango", "swing", "ballroom", "latin", "hip-hop", "contemporary"];
        let mut dance_styles = Vec::new();
        let combined_text = format!("{} {}", title, text_content).to_lowercase();
        for &style in &dance_keywords {
            if combined_text.contains(style) {
                dance_styles.push(style.to_string());
            }
        }
        if dance_styles.is_empty() {
            dance_styles.push("social".to_string());
        }
        
        Some(CreateEventRequest {
            title: format!("[WEB] {}", title),
            description: Some(text_content),
            event_type: Some("social".to_string()),
            dance_styles,
            start_date,
            end_date: None,
            venue_name: Some("Web Venue".to_string()),
            address: Some("TBA".to_string()),
            city: city.to_string(),
            state: state.to_string(),
            zip_code: None,
            latitude: None,
            longitude: None,
            ticket_price: None,
            max_attendees: None,
            image_url: None,
            website_url: None,
            contact_email: None,
            contact_phone: None,
        })
    }
    
    async fn scan_web(&self) -> Vec<CreateEventRequest> {
        println!("Scraping dance websites...");
        let events = self.scrape_dance_websites().await;
        println!("Found {} events from web scraping", events.len());
        events
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("Starting Event Scraper Service...");

    let mut scraper = EventScraper::new("http://localhost:8061".to_string());
    
    // Get authentication token
    scraper.get_remote_token().await?;

    // Scan for events
    println!("Scanning Telegram for events...");
    let telegram_events = scraper.scan_telegram().await;
    
    println!("Scanning web for events...");
    let web_events = scraper.scan_web().await;

    // Submit all found events
    let all_events = [telegram_events, web_events].concat();
    
    println!("\nSubmitting {} events to DanceConnect API...", all_events.len());
    
    for (i, event) in all_events.iter().enumerate() {
        println!("[{}/{}] Submitting: {}", i + 1, all_events.len(), event.title);
        
        match scraper.submit_event(event.clone()).await {
            Ok(_) => println!("✓ Successfully submitted: {}", event.title),
            Err(e) => eprintln!("✗ Failed to submit '{}': {}", event.title, e),
        }
        
        // Rate limiting
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    }

    println!("\n✓ Event scraping and submission completed!");
    println!("Check the DanceConnect dashboard to see the new events.");
    Ok(())
}