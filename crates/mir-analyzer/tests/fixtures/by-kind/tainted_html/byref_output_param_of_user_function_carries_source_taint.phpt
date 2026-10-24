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
//  ^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function branching() {
    readIf($v, true);
    echo $v;
//  ^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
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
//  ^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function appended() {
    $v = 'a';
    readAppend($v);
    echo $v;
//  ^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function arrayValue() {
    readArray($v);
    echo $v['k'];
//  ^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function copiesTaintedArgument() {
    copyIn($_GET['a'], $v);
    echo $v;
//  ^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function copiesCleanArgument() {
    copyIn('lit', $v);
    echo $v;
}

function secondOfTwo() {
    readSecond($a, $b);
    echo $a;
    echo $b;
//  ^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function typed() {
    $s = '';
    readTyped($s);
    echo $s;
//  ^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function namedArgument() {
    read(out: $v);
    echo $v;
//  ^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function methodCall() {
    (new Reader)->read($v);
    echo $v;
//  ^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function staticCall() {
    Reader::sread($v);
    echo $v;
//  ^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function propertyOfTaintedObject() {
    (new Reader)->fromProp($v);
    echo $v;
//  ^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function methodCallsMethod() {
    (new Reader)->nested($v);
    echo $v;
//  ^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function staticCallsStatic() {
    Reader::snested($v);
    echo $v;
//  ^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}
