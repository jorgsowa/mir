===description===
Checking that the host is set or not too long doesn't check which host it is.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
    <MixedOperand errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function sendIfSet(string $to): void {
    $host = $_SERVER['HTTP_HOST'];
    if ($host === '') {
        return;
    }
    mail($to, 'Reset', "https://$host/reset");
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ HostHeaderInjection: Email body built from the request's Host header — possible password-reset link poisoning
}

function sendIfShort(string $to): void {
    $host = $_SERVER['HTTP_HOST'];
    if (strlen($host) > 255) {
        return;
    }
    mail($to, 'Reset', "https://$host/reset");
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ HostHeaderInjection: Email body built from the request's Host header — possible password-reset link poisoning
}
