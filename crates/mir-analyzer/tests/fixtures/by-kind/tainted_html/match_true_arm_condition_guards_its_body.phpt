===description===
An arm condition of `match (true)` guards that arm's body only.
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
function guarded(): void {
    $x = $_GET['x'];
    echo match (true) {
        ctype_digit($x) => $x,
        default => 'n',
    };
}

function otherArm(): void {
    $x = $_GET['x'];
    echo match (true) {
//  ^ +3:6 TaintedHtml: Tainted HTML output — possible XSS
        ctype_digit($x) => 'n',
        default => $x,
    };
}

function notTrue(bool $flag): void {
    $x = $_GET['x'];
    echo match ($flag) {
//  ^ +3:6 TaintedHtml: Tainted HTML output — possible XSS
        ctype_digit($x) => $x,
        default => 'n',
    };
}
