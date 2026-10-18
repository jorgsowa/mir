===description===
Only the `$_SERVER` entries a client controls are tainted: request headers,
the URI and query string, the method and auth credentials. Entries the server
or environment sets, `$_ENV`, CLI arguments and standard input are not. `$GLOBALS['_GET']`
reads the superglobal itself.
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
function requestControlled(string $header): void {
    echo $_SERVER['HTTP_USER_AGENT'];
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    echo $_SERVER['REQUEST_URI'];
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    echo $_SERVER['PHP_SELF'];
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    echo $_SERVER['QUERY_STRING'];
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    echo $_SERVER["HTTP_$header"];
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    echo $GLOBALS['_GET']['q'];
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    foreach ($_SERVER as $value) {
        echo $value;
//      ^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    }
}

function serverControlled(): void {
    echo $_SERVER['DOCUMENT_ROOT'];
    echo $_SERVER['SCRIPT_NAME'];
    echo $_SERVER['REMOTE_ADDR'];
    echo $_SERVER['REQUEST_TIME'];
    echo $_SERVER['APP_ENV'];
    echo $_ENV['APP_SECRET'];
    echo $GLOBALS['config'];
    system('ls ' . $_SERVER['argv'][1]);
    include $_SERVER['DOCUMENT_ROOT'] . '/header.php';
    echo file_get_contents('php://stdin');
}

function readsHeader(): string {
    return (string) $_SERVER['HTTP_REFERER'];
}

function readsRoot(): string {
    return (string) $_SERVER['DOCUMENT_ROOT'];
}

function throughFunctions(): void {
    echo readsHeader();
//  ^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    echo readsRoot();
}
