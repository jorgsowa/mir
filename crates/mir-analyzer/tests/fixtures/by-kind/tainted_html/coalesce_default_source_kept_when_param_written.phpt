===description===
The coalesce rules out the source only while the parameter still holds the argument and no other return reads it.
===config===
<mir>
  <issueHandlers>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedReturnStatement errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function w(?string $x = null): string {
    if (rand(0, 1)) {
        $x = null;
    }
    return $x ?? $_SERVER['REQUEST_URI'];
}
function v(?string $x = null): string {
    if ($x === 'a') {
        return $_SERVER['REQUEST_URI'];
    }
    return $x ?? 'b';
}
function f(): void {
    echo w('static');
//  ^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    echo v('static');
//  ^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}
