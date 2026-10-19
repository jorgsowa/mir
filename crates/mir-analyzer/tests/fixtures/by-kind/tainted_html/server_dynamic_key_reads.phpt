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
    $either = $_GET['x'] ? 'REQUEST_URI' : 'APP_ENV';
    echo $_SERVER[$either];
}

function prefixedKeys(string $name): void {
    echo $_SERVER['HTTP_' . $name];
    echo $_SERVER["HTTP_{$name}"];
    echo $_SERVER[$name . '_HOME'];
}

function nestedAndNullCoalesce(string $key): void {
    echo $_SERVER[$key][0] ?? '';
    echo $_SERVER[$_GET['k']];
}

function wholeSuperglobalStillTainted(): void {
    foreach ($_SERVER as $name => $value) {
        echo $value;
        echo $_SERVER[$name];
    }
}
===expect===
TaintedHtml@8:4-8:25: Tainted HTML output — possible XSS
TaintedHtml@22:4-22:28: Tainted HTML output — possible XSS
TaintedHtml@24:4-24:27: Tainted HTML output — possible XSS
TaintedHtml@28:4-28:35: Tainted HTML output — possible XSS
TaintedHtml@29:4-29:34: Tainted HTML output — possible XSS
TaintedHtml@35:4-35:30: Tainted HTML output — possible XSS
TaintedHtml@40:8-40:20: Tainted HTML output — possible XSS
TaintedHtml@41:8-41:29: Tainted HTML output — possible XSS
