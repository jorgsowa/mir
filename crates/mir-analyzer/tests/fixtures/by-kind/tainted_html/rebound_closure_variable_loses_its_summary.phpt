===description===
A variable bound to a closure summary stops using it once something else is
assigned to it: a foreach value or key, a destructuring target, a by-reference
output argument, or a by-reference capture the enclosing scope rebinds. A
closure that is still bound keeps its summary.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
    <MixedOperand errorLevel="suppress"/>
    <MixedFunctionCall errorLevel="suppress"/>
    <MissingClosureParamType errorLevel="suppress"/>
    <MissingClosureReturnType errorLevel="suppress"/>
    <UnusedClosureParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedForeachValue errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function foreachValue(): void {
    $f = fn ($x) => $x;
    foreach (['htmlspecialchars'] as $f) {
        echo $f($_GET['x']);
    }
}

function foreachKey(): void {
    $f = fn ($x) => $x;
    foreach (['htmlspecialchars' => 1] as $f => $_) {
        echo $f($_GET['x']);
    }
}

function destructured(): void {
    $f = fn ($x) => $x;
    [$f] = ['htmlspecialchars'];
    echo $f($_GET['x']);
}

function destructuredWithKey(): void {
    $f = fn ($x) => $x;
    ['f' => $f] = ['f' => 'htmlspecialchars'];
    echo $f($_GET['x']);
}

function foreachDestructured(): void {
    $f = fn ($x) => $x;
    foreach ([['htmlspecialchars']] as [$f]) {
        echo $f($_GET['x']);
    }
}

function reassigned(): void {
    $f = fn ($x) => $x;
    $f = 'htmlspecialchars';
    echo $f($_GET['x']);
}

function referenceAlias(): void {
    $f = fn ($x) => $x;
    $g = 'htmlspecialchars';
    $f = &$g;
    echo $f($_GET['x']);
}

function outputArgument(): void {
    $f = fn ($x) => $x;
    preg_match('/./', 'a', $f);
    echo $f($_GET['x']);
}

function byReferenceCapture(): void {
    $esc = fn ($x) => $x;
    $run = function ($v) use (&$esc) { return $esc($v); };
    $esc = 'htmlspecialchars';
    echo $run($_GET['x']);
}

function byValueCaptureKeepsSnapshot(): void {
    $esc = fn ($x) => $x;
    $run = function ($v) use ($esc) { return $esc($v); };
    $esc = 'htmlspecialchars';
    echo $run($_GET['x']);
//  ^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function stillBound(): void {
    $f = fn ($x) => $x;
    echo $f($_GET['x']);
//  ^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    foreach ([1, 2] as $i) {
        echo $f($_GET['x']);
//      ^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    }
}

function reboundToClosure(): void {
    $f = fn ($x) => htmlspecialchars($x);
    $f = fn ($x) => $x;
    echo $f($_GET['x']);
//  ^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function reboundToSanitizingClosure(): void {
    $f = fn ($x) => $x;
    $f = fn ($x) => htmlspecialchars($x);
    echo $f($_GET['x']);
}
===expect===
TaintedHtml@65:4-65:26: Tainted HTML output — possible XSS
TaintedHtml@71:4-71:24: Tainted HTML output — possible XSS
TaintedHtml@74:8-74:28: Tainted HTML output — possible XSS
TaintedHtml@82:4-82:24: Tainted HTML output — possible XSS
