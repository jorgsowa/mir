===description===
Under the CLI, `$_SERVER` holds the process environment rather than client data, which is why httpoxy guards read `HTTP_PROXY` only there.
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
function ternary(): void {
    echo in_array(PHP_SAPI, ['cli', 'phpdbg'], true) ? $_SERVER['HTTP_PROXY'] ?? '' : '';
}

function branch(): void {
    if (PHP_SAPI === 'cli') {
        echo $_SERVER['HTTP_PROXY'];
    }
}

function exits(): void {
    if ('cli' !== php_sapi_name()) {
        exit;
    }
    echo $_SERVER['REQUEST_URI'];
}

function elsewhere(): void {
    if (PHP_SAPI === 'cli') {
        echo 'a';
    } else {
        echo $_SERVER['REQUEST_URI'];
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    }
    echo $_SERVER['REQUEST_URI'];
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function otherSapi(): void {
    if (PHP_SAPI === 'fpm-fcgi') {
        echo $_SERVER['REQUEST_URI'];
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    }
    echo in_array(PHP_SAPI, ['cli', 'fpm-fcgi'], true) ? $_SERVER['HTTP_PROXY'] ?? '' : '';
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function otherSuperglobal(): void {
    if (PHP_SAPI === 'cli') {
        echo $_GET['x'];
//      ^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    }
}
