===description===
A configured base URL, a host checked against an allowlist, and the host header
used outside the message body aren't reported.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedOperand errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
const BASE_URL = 'https://example.com';

/** @param list<string> $allowedHosts */
function sendReset(string $email, string $token, array $allowedHosts): void {
    mail($email, 'Password reset', BASE_URL . '/reset?token=' . $token);

    $host = $_SERVER['HTTP_HOST'];
    if (!in_array($host, $allowedHosts, true)) {
        return;
    }
    mail($email, 'Password reset', "https://$host/reset?token=$token");
}

function logged(string $email): void {
    error_log('Mail sent from ' . $_SERVER['HTTP_HOST']);
    mail($email, 'Sent from ' . $_SERVER['HTTP_HOST'], 'Hello');
}
