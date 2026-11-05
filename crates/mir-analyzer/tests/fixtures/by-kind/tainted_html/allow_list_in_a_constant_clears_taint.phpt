===description===
A strict check against an allow-list or a scalar held in a class or global constant clears the
value. A `static::` constant can be overridden and a list holding a non-literal proves nothing.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
const ALLOWED = ['a', 'b'];
const ONE = 'a';

class Q {
    const ALLOWED = ['a', 'b'];
    const ONE = 'a';

    public function classList(): void {
        $x = $_GET['x'];
        if (in_array($x, self::ALLOWED, true)) {
            echo $x;
        }
    }

    public function classListElement(): void {
        if (in_array($_GET['x'], Q::ALLOWED, true)) {
            echo $_GET['x'];
        }
    }

    public function classScalar(): void {
        $x = $_GET['x'];
        if ($x === self::ONE) {
            echo $x;
        }
    }

    public function lateBound(): void {
        $x = $_GET['x'];
        if ($x === static::ONE) {
            echo $x;
//          ^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
        }
    }
}

function globalList(): void {
    $x = $_GET['x'];
    if (in_array($x, ALLOWED, true)) {
        echo $x;
    }
}

function globalScalar(): void {
    $x = $_GET['x'];
    if ($x !== ONE) {
        exit;
    }
    echo $x;
}

function failingPath(): void {
    $x = $_GET['x'];
    if (!in_array($x, ALLOWED, true)) {
        echo $x;
//      ^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    }
}

function loose(): void {
    if (in_array($_GET['x'], ALLOWED)) {
        echo $_GET['x'];
//      ^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    }
}
