===description===
A variable replaced on the failing path of a guard is clean after the merge, because both
paths end up clean. A branch that keeps the raw value still reports.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function defaulted(): void {
    $x = $_GET['x'];
    if (!ctype_digit($x)) {
        $x = '0';
    }
    echo $x;
}

function bothReplaced(bool $c): void {
    $x = $_GET['x'];
    if ($c) {
        $x = 'a';
    } else {
        $x = 'b';
    }
    echo $x;
}

function kept(bool $c): void {
    $x = $_GET['x'];
    if ($c) {
        $x = '0';
    }
    echo $x;
//  ^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function throwingTry(): void {
    $x = $_GET['x'];
    try {
        $x = risky();
    } catch (Exception) {
    }
    echo $x;
//  ^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function risky(): string {
    return 'a';
}

function loop(array $list): void {
    $x = $_GET['x'];
    foreach ($list as $_) {
        $x = 'a';
    }
    echo $x;
//  ^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}
