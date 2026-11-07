===description===
A call to a function or static method that throws or exits unless a parameter passes a guard guards the argument once it returns.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
    <MissingParamType errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
    <MissingThrowsDocblock errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function digits($v): void { if (!ctype_digit($v)) { throw new Exception(); } }
function short($v): void { if (strlen($v) > 9) { throw new Exception(); } }
class V {
    public static function word(string $pre, $v): void { if (preg_match('/^[a-z]+$/D', $v) !== 1) exit(1); }
}

$a = $_GET['a'];
digits($a);
echo $a;

$b = $_GET['b'];
V::word('x', $b);
echo $b;

$c = $_GET['c'];
short($c);
echo $c;
//<^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
