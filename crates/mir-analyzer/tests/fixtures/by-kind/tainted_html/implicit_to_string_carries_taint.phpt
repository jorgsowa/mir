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
}

function stored() {
    $w = new Wrapper($_GET['a']);
    echo $w;
}

function printed() {
    $w = new Wrapper($_GET['a']);
    print $w;
}

function concatenated() {
    $w = new Wrapper($_GET['a']);
    echo '<b>' . $w . '</b>';
}

function interpolated() {
    $w = new Wrapper($_GET['a']);
    echo "<b>$w</b>";
}

function cast() {
    $w = new Wrapper($_GET['a']);
    echo (string) $w;
}

function strvalCall() {
    $w = new Wrapper($_GET['a']);
    echo strval($w);
}

function passThroughBuiltin() {
    $w = new Wrapper($_GET['a']);
    echo trim($w);
}

function setter() {
    $w = new Wrapper('literal');
    $w->setName($_GET['a']);
    echo $w;
}

function inherited() {
    echo new Child($_GET['a']);
}

function readsSource() {
    echo new Request();
}

function readsSourceStored() {
    $r = new Request();
    echo $r;
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
}
===expect===
TaintedHtml@40:4-40:33: Tainted HTML output — possible XSS
TaintedHtml@45:4-45:12: Tainted HTML output — possible XSS
TaintedHtml@50:4-50:12: Tainted HTML output — possible XSS
TaintedHtml@55:4-55:29: Tainted HTML output — possible XSS
TaintedHtml@60:4-60:21: Tainted HTML output — possible XSS
TaintedHtml@65:4-65:21: Tainted HTML output — possible XSS
TaintedHtml@70:4-70:20: Tainted HTML output — possible XSS
TaintedHtml@75:4-75:18: Tainted HTML output — possible XSS
TaintedHtml@81:4-81:12: Tainted HTML output — possible XSS
TaintedHtml@85:4-85:31: Tainted HTML output — possible XSS
TaintedHtml@89:4-89:23: Tainted HTML output — possible XSS
TaintedHtml@94:4-94:12: Tainted HTML output — possible XSS
TaintedHtml@121:4-121:63: Tainted HTML output — possible XSS
