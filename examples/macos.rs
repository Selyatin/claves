fn main() {
    #[cfg(target_os = "macos")]
    {
        use accessibility_sys::{kAXTrustedCheckOptionPrompt, AXIsProcessTrustedWithOptions};
        use claves::{deinit, init};
        use core_foundation::{
            base::{FromVoid, ToVoid},
            dictionary::CFMutableDictionary,
            number::CFNumber,
            string::CFString,
        };
        use std::ffi::c_void;

        /// Checks the Accessibility permission, if not available prompts the user for it.
        unsafe fn check_accessibility_permission() -> bool {
            let mut dict: CFMutableDictionary<CFString, CFNumber> = CFMutableDictionary::new();

            dict.add(
                &CFString::from_void(kAXTrustedCheckOptionPrompt as *const c_void).to_owned(),
                &1i64.into(),
            );

            let app_has_permissions =
                AXIsProcessTrustedWithOptions(dict.into_untyped().to_void() as *const _);

            app_has_permissions
        }

        unsafe {
            if !check_accessibility_permission() {
                // You can re-launch the software here as well, check
                // https://developer.apple.com/forums/thread/703188 for an example way of doing that.
                return;
            }
        }

        let receiver = init();

        println!("Initialized\nWaiting for 5 Events");

        for _ in 0..5 {
            dbg!(receiver.recv().unwrap());
        }

        deinit();

        println!("Deinitialized");
    }
}
