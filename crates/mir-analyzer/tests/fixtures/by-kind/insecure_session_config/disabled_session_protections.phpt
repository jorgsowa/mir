===description===
Turning off strict mode, cookie-only sessions or HttpOnly session cookies, or
turning on `use_trans_sid`, through `ini_set()` or `session_start()` options.
===file===
<?php
function configure(): void {
    ini_set('session.use_strict_mode', '0');
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InsecureSessionConfig: Insecure session handling: session.use_strict_mode is turned off
    ini_set('session.use_only_cookies', 0);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InsecureSessionConfig: Insecure session handling: session.use_only_cookies is turned off
    ini_set('Session.Cookie_HttpOnly', 'Off');
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InsecureSessionConfig: Insecure session handling: session.cookie_httponly is turned off
    ini_set('session.use_trans_sid', true);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InsecureSessionConfig: Insecure session handling: session.use_trans_sid puts the session ID in URLs
    session_start(['cookie_lifetime' => 3600, 'cookie_httponly' => false]);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InsecureSessionConfig: Insecure session handling: session.cookie_httponly is turned off
}
