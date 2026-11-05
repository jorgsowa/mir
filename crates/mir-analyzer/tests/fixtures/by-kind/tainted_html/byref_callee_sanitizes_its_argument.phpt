===description===
A user function that sanitizes a by-ref argument clears it at the call; one that leaves it, or
sanitizes only on one path, doesn't.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
    <MixedOperand errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function escaped(string &$x): void {
    $x = htmlspecialchars($x);
}

function untouched(string &$x): void {
//                 ^^^^^^^^^^ UnusedParam: Parameter $x is never used
    $x .= 'a';
}

function sometimes(string &$x, bool $c): void {
    if ($c) {
        $x = htmlspecialchars($x);
    }
}

function writesSource(string &$x): void {
    $x = $_GET['y'];
}

function callers(bool $c): void {
    $a = $_GET['x'];
    escaped($a);
    echo $a;

    $b = $_GET['x'];
    untouched($b);
    echo $b;
//  ^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS

    $d = $_GET['x'];
    sometimes($d, $c);
    echo $d;
//  ^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS

    $e = 'clean';
    writesSource($e);
    echo $e;
//  ^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

class S {
    public function escape(string &$x): void {
        $x = htmlspecialchars($x);
    }

    public function caller(): void {
        $a = $_GET['x'];
        $this->escape($a);
        echo $a;
    }
}
