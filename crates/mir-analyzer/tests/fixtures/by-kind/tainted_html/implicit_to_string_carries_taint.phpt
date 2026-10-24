===description===
An object used as a string (echo, print, concatenation, interpolation, a string
cast or a pass-through builtin) carries what its `__toString()` returns: a
property the constructor or a setter stored, or source data the method reads
itself. A sanitizing or clean `__toString()` and a clean construction do not.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
    <MixedReturnStatement errorLevel="suppress"/>
    <MissingReturnType errorLevel="suppress"/>
    <MissingParamType errorLevel="suppress"/>
    <MissingPropertyType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Wrapper {
    public function __construct(private string $n, private string $safe = 'fixed') {}

    public function setName(string $n): void {
        $this->n = $n;
    }

    public function __toString(): string {
        return $this->n;
    }
}

class Escaping {
    public function __construct(private string $n) {}

    public function __toString(): string {
        return htmlspecialchars($this->n);
    }
}

class Fixed {
    public function __construct(private string $n, private string $safe = 'fixed') {}

    public function __toString(): string {
        return $this->safe;
    }
}

class Request {
    public function __toString(): string {
        return $_GET['a'];
    }
}

class Child extends Wrapper {
}

function echoed() {
    echo new Wrapper($_GET['a']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function stored() {
    $w = new Wrapper($_GET['a']);
    echo $w;
//  ^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function printed() {
    $w = new Wrapper($_GET['a']);
    print $w;
//  ^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function concatenated() {
    $w = new Wrapper($_GET['a']);
    echo '<b>' . $w . '</b>';
//  ^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function interpolated() {
    $w = new Wrapper($_GET['a']);
    echo "<b>$w</b>";
//  ^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function cast() {
    $w = new Wrapper($_GET['a']);
    echo (string) $w;
//  ^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function strvalCall() {
    $w = new Wrapper($_GET['a']);
    echo strval($w);
//  ^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function passThroughBuiltin() {
    $w = new Wrapper($_GET['a']);
    echo trim($w);
//  ^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function setter() {
    $w = new Wrapper('literal');
    $w->setName($_GET['a']);
    echo $w;
//  ^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function inherited() {
    echo new Child($_GET['a']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function readsSource() {
    echo new Request();
//  ^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function readsSourceStored() {
    $r = new Request();
    echo $r;
//  ^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function cleanConstruction() {
    echo new Wrapper('literal');
}

function sanitizingToString() {
    echo new Escaping($_GET['a']);
}

function otherProperty() {
    echo new Fixed($_GET['a']);
}

function overwritten() {
    $w = new Wrapper($_GET['a']);
    echo strlen((string) $w);
    $w = new Wrapper('literal');
    echo $w;
}

function sanitizedAfterwards() {
    echo htmlspecialchars(new Wrapper($_GET['a']));
}

function unionOfObjects($c) {
    echo $c ? new Wrapper($_GET['a']) : new Wrapper('literal');
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}
