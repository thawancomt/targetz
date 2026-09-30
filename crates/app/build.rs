fn main() {
    #[cfg(windows)]
    {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("../../assets/icon.ico");
        res.set("ProductName", "Targetz");
        res.set("FileDescription", "Whatever Targetz");
        res.set("LegalCopyright", "Copyright (C) 2026");
        let _ = res.compile();
    }
}
