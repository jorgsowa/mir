===description===
Session settings whose name or value is held in a variable with one literal
value are checked like literals.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <PossiblyInvalidArgument errorLevel="suppress"/>
    <PossiblyFalseArgument errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function boot(string $fromConfig): void {
    $off = '0';
    ini_set('session.use_strict_mode', $off);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InsecureSessionConfig: Insecure session handling: session.use_strict_mode is turned off
    $options = ['cookie_httponly' => false];
    session_start($options);
//  ^^^^^^^^^^^^^^^^^^^^^^^ InsecureSessionConfig: Insecure session handling: session.cookie_httponly is turned off
    ini_set('session.use_only_cookies', $fromConfig);
}
