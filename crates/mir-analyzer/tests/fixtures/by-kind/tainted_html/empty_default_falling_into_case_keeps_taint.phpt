===description===
An empty `default:` lets any value into the next case body, so that label proves nothing about it.
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
function variable_subject(): void {
    $x = $_GET['a'];
    switch ($x) {
        default:
        case 'x':
            echo $x;
//          ^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    }
}

function element_subject(): void {
    switch ($_GET['a']) {
        default:
        case 'x':
            echo $_GET['a'];
//          ^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    }
}

function default_after_the_labels(): void {
    $x = $_GET['a'];
    switch ($x) {
        case 'x':
            echo $x;
            break;
        default:
            echo strlen($x);
    }
}
