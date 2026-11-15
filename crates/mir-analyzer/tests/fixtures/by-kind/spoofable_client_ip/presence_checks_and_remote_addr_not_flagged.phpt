===description===
Checking whether a forwarded header is present, deciding on `REMOTE_ADDR`, and a
forwarded IP read only from a trusted proxy aren't reported.
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
/** @param list<string> $proxies */
function clientIp(array $proxies): string {
    $ip = $_SERVER['REMOTE_ADDR'];
    if (in_array($ip, $proxies, true) && isset($_SERVER['HTTP_X_FORWARDED_FOR'])) {
        $ip = $_SERVER['HTTP_X_FORWARDED_FOR'];
    }
    if ($ip === '127.0.0.1') {
        return 'local';
    }
    if (!empty($_SERVER['HTTP_CLIENT_IP']) && $_SERVER['HTTP_CLIENT_IP'] !== '') {
        return 'proxied';
    }
    return (string) $ip;
}
