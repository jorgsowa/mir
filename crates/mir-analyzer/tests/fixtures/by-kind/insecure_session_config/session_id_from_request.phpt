===description===
A session ID taken from a request parameter lets an attacker fix the victim's
session. Setting a generated ID, reading the current one, or passing on the
session cookie (what PHP itself does) is fine.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function resume(): string {
    session_id($_GET['sid']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^ InsecureSessionConfig: Insecure session handling: session_id() takes the ID from a request parameter (session fixation)
    session_id((string) ($_POST['PHPSESSID'] ?? ''));
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InsecureSessionConfig: Insecure session handling: session_id() takes the ID from a request parameter (session fixation)
    session_id(bin2hex(random_bytes(16)));
    session_id($_COOKIE[session_name()]);
    return session_id();
}
