===description===
A variable rebound by `=&` or `unset` no longer takes writes to its old reference partner.
===config===
<mir>
  <issueHandlers>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function f(): void {
    $b = 'c';
    $c = $_GET['x'];
    $a = &$b;
    $a = &$c;
    $b = 'd';
    echo $a;
//  ^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}
function g(): void {
    $b = 'c';
    $a = &$b;
    unset($a);
    $a = $_GET['x'];
    $b = 'd';
    echo $a;
//  ^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}
