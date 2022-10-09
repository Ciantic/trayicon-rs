/// Tray Icon event sender
#[derive(Clone)]
pub(crate) struct TrayIconSender<T>(std::sync::Arc<dyn Fn(&T)>);

impl<T> std::fmt::Debug for TrayIconSender<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TrayIconSender")
            .field("0", &"<function>")
            .finish()
    }
}

impl<T> TrayIconSender<T> {
    pub(crate) fn new(f: impl Fn(&T) + 'static) -> Self {
        TrayIconSender(std::sync::Arc::new(f))
    }
}

impl<T> TrayIconSender<T>
where
    T: PartialEq + Clone + 'static,
{
    pub fn send(&self, e: &T) {
        self.0(e)
    }
}
