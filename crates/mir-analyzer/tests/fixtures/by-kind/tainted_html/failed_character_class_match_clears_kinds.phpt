===description===
A failed match of a single character class proves the characters don't occur, as when WordPress
returns the raw value only if it has no markup characters. A successful match proves nothing,
and neither does a failed match of a longer pattern.
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
function esc(string $text): string {
    if (!preg_match('/[&<>"\']/', $text)) {
        return $text;
    }
    return htmlspecialchars($text);
}

function callers(): void {
    echo esc($_GET['x']);
}

function failed(): void {
    $x = $_GET['x'];
    if (!preg_match('/[<>"\'&]/', $x)) {
        echo $x;
    }
}

function matched(): void {
    $x = $_GET['x'];
    if (preg_match('/[<>"\'&]/', $x)) {
        echo $x;
//      ^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    }
}

function otherCharacters(): void {
    $x = $_GET['x'];
    if (!preg_match('/[<>]/', $x)) {
        echo $x;
//      ^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    }
}

function longerPattern(): void {
    $x = $_GET['x'];
    if (!preg_match('/<script/', $x)) {
        echo $x;
//      ^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    }
}
