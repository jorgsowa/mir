===description===
`unset($a['x'])` removes that element's taint; the others keep theirs.
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
function removed(): void {
    $a = ['x' => $_GET['x'], 'y' => 'c'];
    unset($a['x']);
    echo implode(',', $a);
}

function other(): void {
    $a = ['x' => $_GET['x'], 'y' => $_GET['y']];
    unset($a['x']);
    echo $a['y'];
//  ^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}
