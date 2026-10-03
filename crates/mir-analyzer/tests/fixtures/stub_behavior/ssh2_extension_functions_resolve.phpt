===description===
FP-I1: the `ssh2` PECL extension (ssh2_connect, ssh2_auth_password, ...)
had no vendored stubs/ dir despite PhpStormStubsMap.php already listing
every entry — same missing-stub root cause as the fixed C6 (ast).
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
    <MixedArgument errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php

function connect(string $host, string $user, string $password) {
//       ^^^^^^^ MissingReturnType: Function connect() has no return type annotation
    $session = ssh2_connect($host);
    ssh2_auth_password($session, $user, $password);
    return $session;
}
===expect===
