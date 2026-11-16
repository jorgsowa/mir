===description===
A session ID taken from a GET or POST parameter fixes the session, also through
a variable, `trim()` or `filter_input()`; one read from the session cookie, or
generated, doesn't.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
    <PossiblyInvalidArgument errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function fromVariable(): void {
    $sid = $_GET['sid'];
    session_id($sid);
//  ^^^^^^^^^^^^^^^^ InsecureSessionConfig: Insecure session handling: session_id() takes the ID from a request parameter (session fixation)
}

function trimmed(): void {
    session_id(trim($_POST['sid']));
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InsecureSessionConfig: Insecure session handling: session_id() takes the ID from a request parameter (session fixation)
}

function filtered(): void {
    session_id((string) filter_input(INPUT_GET, 'sid'));
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InsecureSessionConfig: Insecure session handling: session_id() takes the ID from a request parameter (session fixation)
}

function fromCookie(): void {
    $sid = $_COOKIE['sid'];
    session_id($sid);
}

function generated(): void {
    session_id(session_create_id());
}
