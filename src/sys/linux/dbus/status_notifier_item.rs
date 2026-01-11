use std::sync::{Arc, Mutex};
use zbus::interface;
use zbus::object_server::SignalEmitter;

#[allow(dead_code)]
pub enum StatusNotifierEvent {
    /// Activated at the given coordinates (x, y), equivalent of left-click
    Activate(i32, i32),
    /// Context menu requested at the given coordinates (x, y)
    ContextMenu(i32, i32),
    /// XDG activation token provided
    ProvideXdgActivationToken(String),
    /// Scrolled with the given delta and orientation
    Scroll(i32, String),
    /// Secondary activation (e.g., middle-click) at the given coordinates (x, y)
    SecondaryActivate(i32, i32),
}

#[derive(Debug, Clone)]
pub struct IconData {
    pub buffer: Option<Vec<u8>>,
    pub width: u32,
    pub height: u32,
}

// Minimal in-process implementation of `org.kde.StatusNotifierItem` to register
#[derive(Debug)]
pub struct StatusNotifierItemImpl {
    pub id: String,
    pub channel_sender: std::sync::mpsc::Sender<StatusNotifierEvent>,
    pub icon_data: Arc<Mutex<IconData>>,
    pub tooltip: Arc<Mutex<String>>,
}

#[interface(name = "org.kde.StatusNotifierItem")]
impl StatusNotifierItemImpl {
    /// Activate method
    pub fn activate(&self, x: i32, y: i32) -> zbus::fdo::Result<()> {
        let _ = self
            .channel_sender
            .send(StatusNotifierEvent::Activate(x, y));
        Ok(())
    }

    /// ContextMenu method
    pub fn context_menu(&self, x: i32, y: i32) -> zbus::fdo::Result<()> {
        let _ = self
            .channel_sender
            .send(StatusNotifierEvent::ContextMenu(x, y));
        Ok(())
    }

    /// ProvideXdgActivationToken method
    pub fn provide_xdg_activation_token(&self, token: &str) -> zbus::fdo::Result<()> {
        let _ = self
            .channel_sender
            .send(StatusNotifierEvent::ProvideXdgActivationToken(
                token.to_string(),
            ));
        Ok(())
    }

    /// Scroll method
    pub fn scroll(&self, delta: i32, orientation: &str) -> zbus::fdo::Result<()> {
        let _ = self
            .channel_sender
            .send(StatusNotifierEvent::Scroll(delta, orientation.to_string()));
        Ok(())
    }

    /// SecondaryActivate method
    pub fn secondary_activate(&self, x: i32, y: i32) -> zbus::fdo::Result<()> {
        let _ = self
            .channel_sender
            .send(StatusNotifierEvent::SecondaryActivate(x, y));
        Ok(())
    }

    /// AttentionIconName property
    #[zbus(property)]
    pub fn attention_icon_name(&self) -> zbus::fdo::Result<String> {
        Ok(String::new())
    }

    /// AttentionIconPixmap property
    #[zbus(property)]
    pub fn attention_icon_pixmap(&self) -> zbus::fdo::Result<Vec<(i32, i32, Vec<u8>)>> {
        Ok(vec![])
    }

    /// AttentionMovieName property
    #[zbus(property)]
    pub fn attention_movie_name(&self) -> zbus::fdo::Result<String> {
        Ok(String::new())
    }

    /// Category property
    #[zbus(property)]
    pub fn category(&self) -> zbus::fdo::Result<String> {
        println!("category() called");
        Ok(String::from("ApplicationStatus"))
    }

    /// IconName property
    #[zbus(property)]
    pub fn icon_name(&self) -> zbus::fdo::Result<String> {
        println!("icon_name() called");
        // Return empty string so that icon_pixmap is used instead
        // If we return a theme icon name, it will be preferred over the pixmap
        if let Ok(icon_data) = self.icon_data.lock() {
            if icon_data.buffer.is_some() {
                return Ok(String::new());
            }
        }
        Ok(String::from("application-x-executable"))
    }

    /// IconPixmap property
    #[zbus(property)]
    pub fn icon_pixmap(&self) -> zbus::fdo::Result<Vec<(i32, i32, Vec<u8>)>> {
        println!("icon_pixmap() called");

        // If we have icon data, use it
        if let Ok(icon_data) = self.icon_data.lock() {
            if let Some(ref buffer) = icon_data.buffer {
                let width = icon_data.width as i32;
                let height = icon_data.height as i32;

                // Convert RGBA to ARGB if needed
                let mut argb_pixmap = Vec::with_capacity(buffer.len());
                for chunk in buffer.chunks(4) {
                    if chunk.len() == 4 {
                        argb_pixmap.push(chunk[3]); // Alpha
                        argb_pixmap.push(chunk[0]); // Red
                        argb_pixmap.push(chunk[1]); // Green
                        argb_pixmap.push(chunk[2]); // Blue
                    }
                }

                return Ok(vec![(width, height, argb_pixmap)]);
            }
        }

        // Fallback: Create a simple 24x24 ARGB pixmap (red square with alpha channel)
        let width = 24i32;
        let height = 24i32;
        let mut pixmap = Vec::with_capacity((width * height * 4) as usize);

        for _y in 0..height {
            for _x in 0..width {
                // ARGB: Alpha, Red, Green, Blue
                pixmap.push(255); // Alpha (opaque)
                pixmap.push(255); // Red
                pixmap.push(100); // Green
                pixmap.push(100); // Blue
            }
        }

        Ok(vec![(width, height, pixmap)])
    }

    /// IconThemePath property
    #[zbus(property)]
    pub fn icon_theme_path(&self) -> zbus::fdo::Result<String> {
        Ok(String::new())
    }

    /// Id property
    #[zbus(property)]
    pub fn id(&self) -> zbus::fdo::Result<String> {
        println!("id() called");
        Ok(self.id.clone())
    }

    /// ItemIsMenu property
    #[zbus(property)]
    pub fn item_is_menu(&self) -> zbus::fdo::Result<bool> {
        Ok(false)
    }

    /// Menu property
    #[zbus(property)]
    pub fn menu(&self) -> zbus::fdo::Result<zbus::zvariant::OwnedObjectPath> {
        Ok(
            zbus::zvariant::OwnedObjectPath::try_from("/MenuBar").map_err(|_| {
                zbus::fdo::Error::UnknownProperty("Failed to create object path".to_string())
            })?,
        )
    }

    /// OverlayIconName property
    #[zbus(property)]
    pub fn overlay_icon_name(&self) -> zbus::fdo::Result<String> {
        Ok(String::from("help-about"))
    }

    /// OverlayIconPixmap property
    #[zbus(property)]
    pub fn overlay_icon_pixmap(&self) -> zbus::fdo::Result<Vec<(i32, i32, Vec<u8>)>> {
        Ok(vec![])
    }

    /// Status property
    #[zbus(property)]
    pub fn status(&self) -> zbus::fdo::Result<String> {
        println!("status() called");
        Ok(String::from("Active"))
    }

    /// Title property
    #[zbus(property)]
    pub fn title(&self) -> zbus::fdo::Result<String> {
        Ok(String::from("Example App"))
    }

    /// ToolTip property
    #[zbus(property)]
    #[allow(clippy::type_complexity)]
    pub fn tool_tip(
        &self,
    ) -> zbus::fdo::Result<(String, Vec<(i32, i32, Vec<u8>)>, String, String)> {
        let tooltip = self.tooltip.lock().unwrap().clone();
        Ok((String::new(), vec![], tooltip, String::new()))
    }

    /// WindowId property
    #[zbus(property)]
    pub fn window_id(&self) -> zbus::fdo::Result<i32> {
        Ok(0)
    }

    // Signals -----------------------------------------------------------

    /// NewAttentionIcon signal
    #[zbus(signal)]
    pub async fn new_attention_icon(ctxt: &SignalEmitter<'_>) -> zbus::Result<()>;

    /// NewIcon signal
    #[zbus(signal)]
    pub async fn new_icon(ctxt: &SignalEmitter<'_>) -> zbus::Result<()>;

    /// NewMenu signal
    #[zbus(signal)]
    pub async fn new_menu(ctxt: &SignalEmitter<'_>) -> zbus::Result<()>;

    /// NewOverlayIcon signal
    #[zbus(signal)]
    pub async fn new_overlay_icon(ctxt: &SignalEmitter<'_>) -> zbus::Result<()>;

    /// NewStatus signal
    #[zbus(signal)]
    pub async fn new_status(ctxt: &SignalEmitter<'_>, _status: &str) -> zbus::Result<()>;

    /// NewTitle signal
    #[zbus(signal)]
    pub async fn new_title(ctxt: &SignalEmitter<'_>) -> zbus::Result<()>;

    /// NewToolTip signal
    #[zbus(signal)]
    pub async fn new_tool_tip(ctxt: &SignalEmitter<'_>) -> zbus::Result<()>;
}
