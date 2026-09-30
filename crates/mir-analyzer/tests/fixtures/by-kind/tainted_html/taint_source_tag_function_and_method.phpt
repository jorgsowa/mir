===description===
New @taint-source docblock tag: marks a function/method's return value
as tainted (attacker-controlled) at every call site, mirroring the
existing @taint-sink mechanism from the source side. Previously no
mechanism existed for this at all -- any function-call result was
unconditionally untainted.
===config===
suppress=UnusedParam,MissingConstructor,MixedReturnStatement,MixedArrayAccess
===file===
<?php
/** @taint-source */
function readUserInput(): string {
    return $_GET['x'] ?? '';
}

echo readUserInput();
//<^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS

class Request {
    /** @taint-source */
    public function getParam(string $name): string {
        return $_GET[$name] ?? '';
    }
}

function handle(Request $req): void {
    echo $req->getParam('x');
//  ^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}
===expect===
