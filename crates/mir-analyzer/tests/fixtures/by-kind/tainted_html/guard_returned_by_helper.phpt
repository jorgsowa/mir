===description===
A function or static method whose body returns a guard on a parameter guards the argument passed for it.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
    <MissingParamType errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function digits($v): bool { return ctype_digit($v); }
function long($v): bool { return strlen($v) > 2; }
class V {
    const ID = '/^[a-f0-9-]+$/D';
    public static function id(string $pre, string $v): bool { return preg_match(self::ID, $v) === 1; }
}

$a = $_GET['a'];
if (digits($a)) { echo $a; }
if (!digits($a)) { exit; }
echo $a;

$b = $_GET['b'];
if (V::id("x", $b)) { echo $b; }

$c = $_GET['c'];
if (long($c)) { echo $c; }
//              ^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
