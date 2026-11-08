===description===
`return $param ?? <source>` returns the source only when the argument is null or omitted.
===config===
<mir>
  <issueHandlers>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedReturnStatement errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function u(?string $x = null): string {
    return $x ?? $_SERVER['REQUEST_URI'];
}
function f(string $s): void {
    echo u('static');
    echo u($s);
}
function g(?string $s): void {
    echo u();
//  ^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    echo u($s);
//  ^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}
