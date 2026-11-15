===description===
An IP from a header the client sets decides an allowlist check, so sending
`X-Forwarded-For: 127.0.0.1` passes it.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function isAdmin(): bool {
    $ip = $_SERVER['HTTP_X_FORWARDED_FOR'] ?? $_SERVER['REMOTE_ADDR'];
    if (in_array($ip, ['10.0.0.1', '10.0.0.2'], true)) {
//               ^^^ SpoofableClientIp: Client IP from a header the client can forge decides a check — possible IP allowlist bypass
        return true;
    }
    return false;
}

function internal(): bool {
    return $_SERVER['HTTP_CLIENT_IP'] === '127.0.0.1' ? true : false;
//         ^^^^^^^^^^^^^^^^^^^^^^^^^^ SpoofableClientIp: Client IP from a header the client can forge decides a check — possible IP allowlist bypass
}

function office(): void {
    $real = trim((string) getenv('HTTP_X_REAL_IP'));
    if (!str_starts_with($real, '192.168.')) {
//                       ^^^^^ SpoofableClientIp: Client IP from a header the client can forge decides a check — possible IP allowlist bypass
        exit;
    }
}
