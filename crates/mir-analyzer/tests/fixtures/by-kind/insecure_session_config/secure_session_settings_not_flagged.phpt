===description===
Enabling the protections, unrelated settings, and values only known at runtime
are not reported.
===file===
<?php
function configure(string $mode): void {
    ini_set('session.use_strict_mode', '1');
    ini_set('session.cookie_httponly', true);
    ini_set('session.use_trans_sid', 0);
    ini_set('session.cookie_secure', '0');
    ini_set('session.use_strict_mode', $mode);
    ini_set('display_errors', '0');
    session_start(['use_strict_mode' => true, 'cookie_samesite' => 'Lax']);
    session_start();
}
