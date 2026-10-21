===description===
A user function or method that writes source data into a by-ref parameter
taints the variable passed for it. A clean, escaped or overwritten write, and
an argument the callee only copies, stay as the arguments' own taint.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
    <MixedOperand errorLevel="suppress"/>
    <MissingParamType errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
    <MissingReturnType errorLevel="suppress"/>
    <MissingPropertyType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function read(&$out) { $out = $_GET['a']; }
function readIf(&$out, $c) { if ($c) { $out = $_GET['a']; } else { $out = 'x'; } }
function readClean(&$out) { $out = 'safe'; }
function readEscaped(&$out) { $out = htmlspecialchars($_GET['a']); }
function readThenClean(&$out) { $out = $_GET['a']; $out = 'x'; }
function readEarlyReturn(&$out, $c) { if ($c) { $out = $_GET['a']; return; } $out = 'x'; }
function readAppend(&$out) { $out .= $_GET['a']; }
function readArray(&$out) { $out = ['k' => $_GET['a']]; }
function copyIn($in, &$out) { $out = $in; }
function readSecond(&$first, &$second) { $first = 'x'; $second = $_GET['a']; }
function readTyped(string &$out): void { $out = $_GET['a']; }

class Reader {
    public $prop;
    public function __construct() { $this->prop = $_GET['a']; }
    public function read(&$out) { $out = $_GET['a']; }
    public static function sread(&$out) { $out = $_GET['a']; }
    public function fromProp(&$out) { $out = $this->prop; }
    public function nested(&$out) { $this->read($out); }
    public static function snested(&$out) { self::sread($out); }
}

function direct() {
    read($v);
    echo $v;
}

function branching() {
    readIf($v, true);
    echo $v;
}

function cleanWrite() {
    readClean($v);
    echo $v;
}

function escapedWrite() {
    readEscaped($v);
    echo $v;
}

function overwrittenWrite() {
    readThenClean($v);
    echo $v;
}

function earlyReturn() {
    readEarlyReturn($v, true);
    echo $v;
}

function appended() {
    $v = 'a';
    readAppend($v);
    echo $v;
}

function arrayValue() {
    readArray($v);
    echo $v['k'];
}

function copiesTaintedArgument() {
    copyIn($_GET['a'], $v);
    echo $v;
}

function copiesCleanArgument() {
    copyIn('lit', $v);
    echo $v;
}

function secondOfTwo() {
    readSecond($a, $b);
    echo $a;
    echo $b;
}

function typed() {
    $s = '';
    readTyped($s);
    echo $s;
}

function namedArgument() {
    read(out: $v);
    echo $v;
}

function methodCall() {
    (new Reader)->read($v);
    echo $v;
}

function staticCall() {
    Reader::sread($v);
    echo $v;
}

function propertyOfTaintedObject() {
    (new Reader)->fromProp($v);
    echo $v;
}

function methodCallsMethod() {
    (new Reader)->nested($v);
    echo $v;
}

function staticCallsStatic() {
    Reader::snested($v);
    echo $v;
}
===expect===
TaintedHtml@26:4-26:12: Tainted HTML output — possible XSS
TaintedHtml@31:4-31:12: Tainted HTML output — possible XSS
TaintedHtml@51:4-51:12: Tainted HTML output — possible XSS
TaintedHtml@57:4-57:12: Tainted HTML output — possible XSS
TaintedHtml@62:4-62:17: Tainted HTML output — possible XSS
TaintedHtml@67:4-67:12: Tainted HTML output — possible XSS
TaintedHtml@78:4-78:12: Tainted HTML output — possible XSS
TaintedHtml@84:4-84:12: Tainted HTML output — possible XSS
TaintedHtml@89:4-89:12: Tainted HTML output — possible XSS
TaintedHtml@94:4-94:12: Tainted HTML output — possible XSS
TaintedHtml@99:4-99:12: Tainted HTML output — possible XSS
TaintedHtml@104:4-104:12: Tainted HTML output — possible XSS
TaintedHtml@109:4-109:12: Tainted HTML output — possible XSS
TaintedHtml@114:4-114:12: Tainted HTML output — possible XSS
