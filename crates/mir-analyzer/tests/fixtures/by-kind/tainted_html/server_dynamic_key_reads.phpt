===description===
A `$_SERVER[$key]` read with a key unknown statically carries the taint of the
key, not of the whole superglobal, unless the key can name a request entry.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
    <MixedOperand errorLevel="suppress"/>
    <MixedReturnStatement errorLevel="suppress"/>
    <MixedArrayOffset errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function env(string $key): mixed {
    return $_SERVER[$key] ?? null;
}

function callers(): void {
    echo env('HOME');
    echo env($_GET['k']);
//  ^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

class Reader {
    private string $envKey = 'APP_ENV';
    public function read(): void {
        echo $_SERVER[$this->envKey];
    }
}

function literalKeyInVariable(): void {
    $safe = 'DOCUMENT_ROOT';
    $request = 'HTTP_HOST';
    echo $_SERVER[$safe];
    echo $_SERVER[$request];
//  ^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    $either = $_GET['x'] ? 'REQUEST_URI' : 'APP_ENV';
    echo $_SERVER[$either];
//  ^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function prefixedKeys(string $name): void {
    echo $_SERVER['HTTP_' . $name];
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    echo $_SERVER["HTTP_{$name}"];
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    echo $_SERVER[$name . '_HOME'];
}

function nestedAndNullCoalesce(string $key): void {
    echo $_SERVER[$key][0] ?? '';
    echo $_SERVER[$_GET['k']];
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function wholeSuperglobalStillTainted(): void {
    foreach ($_SERVER as $name => $value) {
        echo $value;
//      ^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
        echo $_SERVER[$name];
//      ^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    }
}
