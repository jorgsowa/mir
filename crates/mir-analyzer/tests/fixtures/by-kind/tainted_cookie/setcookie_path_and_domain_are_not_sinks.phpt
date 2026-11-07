===description===
PHP rejects separators in a cookie's path and domain, so only its name and value are sinks.
===config===
<mir>
  <issueHandlers>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedArgument errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function test(): void {
    setcookie('id', 'v', 0, $_SERVER['REQUEST_URI'], $_SERVER['HTTP_HOST']);
    setrawcookie('id', 'v', ['path' => $_GET['p'], 'domain' => $_GET['d']]);
    setcookie($_GET['n'], 'v');
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedCookie: Tainted cookie — possible cookie injection
}
