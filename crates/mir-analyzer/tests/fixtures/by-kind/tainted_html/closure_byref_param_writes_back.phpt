===description===
A closure held in a variable leaves in its by-ref argument what it writes back, called directly or through `array_walk`.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
    <MissingClosureParamType errorLevel="suppress"/>
    <MissingClosureReturnType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
$int = function (&$v) { $v = (int) $v; };
$trim = function (&$v) { $v = trim($v); };

$a = $_GET['a'];
$int($a);
echo $a;

$b = $_GET['b'];
$trim($b);
echo $b;
//<^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS

$c = (array) $_GET['c'];
array_walk($c, $int);
echo $c[0];
