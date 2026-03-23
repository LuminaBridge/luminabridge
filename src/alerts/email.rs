//! Email notification module
//! 
//! Sends alert notifications via SMTP email

use crate::error::{Result, Error};
use lettre::{
    Message, SmtpTransport, Transport,
    message::{header::ContentType, MultiPart, SinglePart},
    transport::smtp::authentication::Credentials,
};
use std::time::Duration;

/// Email sender configuration
pub struct EmailSender {
    smtp_host: String,
    smtp_port: u16,
    username: String,
    password: String,
}

impl EmailSender {
    /// Create a new EmailSender
    pub fn new(smtp_host: &str, smtp_port: u16, username: &str, password: &str) -> Self {
        Self {
            smtp_host: smtp_host.to_string(),
            smtp_port,
            username: username.to_string(),
            password: password.to_string(),
        }
    }

    /// Send an email to multiple recipients
    pub async fn send(&self, recipients: &[String], subject: &str, body: &str) -> Result<()> {
        if recipients.is_empty() {
            return Err(Error::Internal("No email recipients configured".to_string()));
        }

        // Create email message
        let mut email_builder = Message::builder()
            .from(
                format!("LuminaBridge Alerts <{}>", self.username)
                    .parse()
                    .map_err(|e| Error::Internal(format!("Invalid sender email: {}", e)))?,
            )
            .subject(subject);

        // Add recipients
        for recipient in recipients {
            email_builder = email_builder.to(
                recipient
                    .parse()
                    .map_err(|e| Error::Internal(format!("Invalid recipient email: {}", e)))?,
            );
        }

        // Create multipart message with plain text and HTML
        let email = email_builder
            .multipart(
                MultiPart::alternative()
                    .singlepart(
                        SinglePart::builder()
                            .header(ContentType::TEXT_PLAIN)
                            .body(body.to_string()),
                    )
                    .singlepart(
                        SinglePart::builder()
                            .header(ContentType::TEXT_HTML)
                            .body(format!(
                                r#"
                                <html>
                                    <body style="font-family: Arial, sans-serif; line-height: 1.6; color: #333;">
                                        <div style="max-width: 600px; margin: 0 auto; padding: 20px;">
                                            <h2 style="color: #2563eb;">🌉 LuminaBridge Alert</h2>
                                            <div style="background-color: #f3f4f6; padding: 15px; border-radius: 8px; margin: 15px 0;">
                                                <h3 style="margin-top: 0; color: #1f2937;">{}</h3>
                                                <p style="white-space: pre-wrap;">{}</p>
                                            </div>
                                            <p style="color: #6b7280; font-size: 14px;">
                                                This is an automated alert from LuminaBridge.
                                            </p>
                                        </div>
                                    </body>
                                </html>
                                "#,
                                subject, body
                            )),
                    ),
            )
            .map_err(|e| Error::Internal(format!("Failed to build email: {}", e)))?;

        // Create SMTP transport
        let creds = Credentials::new(self.username.clone(), self.password.clone());
        
        let mailer = SmtpTransport::relay(&self.smtp_host)
            .map_err(|e| Error::Internal(format!("Failed to create SMTP relay: {}", e)))?
            .port(self.smtp_port)
            .credentials(creds)
            .timeout(Some(Duration::from_secs(30)))
            .build();

        // Send email
        match mailer.send(&email) {
            Ok(_) => {
                tracing::info!("Email alert sent successfully to {} recipients", recipients.len());
                Ok(())
            }
            Err(e) => {
                Err(Error::Internal(format!("Failed to send email: {}", e)))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_sender_creation() {
        let sender = EmailSender::new("smtp.example.com", 587, "user@example.com", "password");
        assert_eq!(sender.smtp_host, "smtp.example.com");
        assert_eq!(sender.smtp_port, 587);
    }
}
