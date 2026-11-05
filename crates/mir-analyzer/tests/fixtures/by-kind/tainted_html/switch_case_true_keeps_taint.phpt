===description===
`switch` compares loosely, so any truthy string enters `case true` and its taint survives.
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
function label_alone(): void {
    $x = $_GET['a'];
    switch ($x) {
        case true:
            echo $x;
//          ^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    }
}

function with_a_literal_label(): void {
    $x = $_GET['a'];
    switch ($x) {
        case true:
        case 'x':
            echo $x;
//          ^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    }
}

function literal_labels_still_clear(): void {
    $x = $_GET['a'];
    switch ($x) {
        case 'x':
            echo $x;
    }
}
