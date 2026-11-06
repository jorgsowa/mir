===description===
`array_walk` with a closure taking the value by reference leaves in the array what the closure writes back.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
    <MissingClosureParamType errorLevel="suppress"/>
    <MissingClosureReturnType errorLevel="suppress"/>
    <UnusedClosureParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
$a = (array) $_GET['a'];
array_walk($a, function (&$v) { $v = (int) $v; });
echo $a[0];

$b = (array) $_GET['b'];
array_walk($b, function (&$v) { $v = trim($v); });
echo $b[0];
//<^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS

function f(array $c): void {
    array_walk($c, function (&$v) { $v = $_GET['c']; });
    echo $c[0];
//  ^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}
