===description===
A `/u` match returns false on invalid UTF-8, so its failure says nothing about the characters.
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
function failed_match_in_branch(): void {
    $x = $_GET['x'];
    if (!preg_match('/[^a-z0-9]/u', $x)) {
        echo $x;
//      ^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    }
}

function failed_match_after_return(): void {
    $x = $_GET['x'];
    if (preg_match('/[^a-z0-9]/u', $x)) {
        return;
    }
    echo $x;
//  ^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function byte_pattern_still_clears(): void {
    $x = $_GET['x'];
    if (!preg_match('/[^a-z0-9]/', $x)) {
        echo $x;
    }
}
