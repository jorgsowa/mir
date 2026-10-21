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
}

function callablePassThrough() {
    $f = strtoupper(...);
    echo $f($_GET['a']);
}

function callableSink() {
    $f = show(...);
    $f($_GET['a']);
}

function callableBuiltinSink() {
    $f = system(...);
    $f($_GET['a']);
}

function callableBuiltinSinkClean() {
    $f = system(...);
    $f('ls');
}

function callableCopied() {
    $f = id(...);
    $g = $f;
    echo $g($_GET['a']);
}

function callableRebound() {
    $f = id(...);
    $f = constant(...);
    echo $f($_GET['a']);
}

function callableThroughCallUserFunc() {
    $f = id(...);
    echo call_user_func($f, $_GET['a']);
}

function propertyClosure() {
    $h = new Holder();
    $h->f = fn($x) => $x;
    echo ($h->f)($_GET['a']);
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
}

function propertyThroughCallUserFunc() {
    $h = new Holder();
    $h->f = fn($x) => $x;
    echo call_user_func($h->f, $_GET['a']);
}

function methodsOfHolder() {
    $h = new Holder();
    echo $h->viaThis($_GET['a']);
    $h->viaThisSink($_GET['a']);
    echo $h->viaThisClean($_GET['a']);
}
===expect===
TaintedHtml@42:4-42:24: Tainted HTML output — possible XSS
TaintedHtml@62:4-62:24: Tainted HTML output — possible XSS
TaintedHtml@67:4-67:24: Tainted HTML output — possible XSS
TaintedHtml@72:4-72:18: Tainted HTML output — possible XSS
TaintedShell@77:4-77:18: Tainted shell command — possible command injection
TaintedHtml@88:4-88:24: Tainted HTML output — possible XSS
TaintedHtml@99:4-99:40: Tainted HTML output — possible XSS
TaintedHtml@105:4-105:29: Tainted HTML output — possible XSS
TaintedHtml@119:4-119:23: Tainted HTML output — possible XSS
TaintedHtml@145:4-145:29: Tainted HTML output — possible XSS
TaintedHtml@151:4-151:43: Tainted HTML output — possible XSS
TaintedHtml@156:4-156:33: Tainted HTML output — possible XSS
TaintedHtml@157:4-157:31: Tainted HTML output — possible XSS
