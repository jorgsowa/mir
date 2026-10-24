===description===
A first-class callable of a named function (`id(...)`) and a closure held in a
property of a local object or `$this` move taint like the function or closure
they stand for: results, sanitizers and sinks inside them. Rebinding the
variable or property drops the old target.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
    <MissingReturnType errorLevel="suppress"/>
    <MissingParamType errorLevel="suppress"/>
    <MissingPropertyType errorLevel="suppress"/>
    <MixedFunctionCall errorLevel="suppress"/>
    <MixedMethodCall errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
    <MissingClosureReturnType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function id($x) {
    return $x;
}

function constant($x) {
    return 'literal';
}

function show($x) {
    echo $x;
}

class Holder {
    public $f;
    public $g;

    public function __construct() {
        $this->f = fn($x) => $x;
    }

    public function viaThis($v) {
        $this->g = fn($x) => $x;
        return ($this->g)($v);
    }

    public function viaThisSink($v) {
        $this->g = function ($x) {
            echo $x;
        };
        ($this->g)($v);
    }

    public function viaThisClean($v) {
        $this->g = fn($x) => 'literal';
        return ($this->g)($v);
    }
}

function callableResult() {
    $f = id(...);
    echo $f($_GET['a']);
//  ^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function callableClean() {
    $f = id(...);
    echo $f('literal');
}

function callableConstant() {
    $f = constant(...);
    echo $f($_GET['a']);
}

function callableSanitizer() {
    $f = htmlspecialchars(...);
    echo $f($_GET['a']);
}

function callableOtherSanitizer() {
    $f = addslashes(...);
    echo $f($_GET['a']);
//  ^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function callablePassThrough() {
    $f = strtoupper(...);
    echo $f($_GET['a']);
//  ^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function callableSink() {
    $f = show(...);
    $f($_GET['a']);
//  ^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function callableBuiltinSink() {
    $f = system(...);
    $f($_GET['a']);
//  ^^^^^^^^^^^^^^ TaintedShell: Tainted shell command — possible command injection
}

function callableBuiltinSinkClean() {
    $f = system(...);
    $f('ls');
}

function callableCopied() {
    $f = id(...);
    $g = $f;
    echo $g($_GET['a']);
//  ^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function callableRebound() {
    $f = id(...);
    $f = constant(...);
    echo $f($_GET['a']);
}

function callableThroughCallUserFunc() {
    $f = id(...);
    echo call_user_func($f, $_GET['a']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function propertyClosure() {
    $h = new Holder();
    $h->f = fn($x) => $x;
    echo ($h->f)($_GET['a']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function propertyClosureClean() {
    $h = new Holder();
    $h->f = fn($x) => 'literal';
    echo ($h->f)($_GET['a']);
}

function propertyClosureSink() {
    $h = new Holder();
    $h->f = function ($x) {
        echo $x;
    };
    ($h->f)($_GET['a']);
//  ^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function propertyClosureRebound() {
    $h = new Holder();
    $h->f = fn($x) => $x;
    $h->f = fn($x) => 'literal';
    echo ($h->f)($_GET['a']);
}

function propertyObjectRebound() {
    $h = new Holder();
    $h->f = fn($x) => $x;
    $h = new Holder();
    echo ($h->f)($_GET['a']);
}

function propertyOtherProperty() {
    $h = new Holder();
    $h->f = fn($x) => $x;
    echo ($h->g)($_GET['a']);
}

function propertyCallable() {
    $h = new Holder();
    $h->f = id(...);
    echo ($h->f)($_GET['a']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function propertyThroughCallUserFunc() {
    $h = new Holder();
    $h->f = fn($x) => $x;
    echo call_user_func($h->f, $_GET['a']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function methodsOfHolder() {
    $h = new Holder();
    echo $h->viaThis($_GET['a']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    $h->viaThisSink($_GET['a']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    echo $h->viaThisClean($_GET['a']);
}
