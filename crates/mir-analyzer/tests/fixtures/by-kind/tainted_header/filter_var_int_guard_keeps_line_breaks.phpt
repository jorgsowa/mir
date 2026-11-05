===description===
`FILTER_VALIDATE_INT` trims whitespace before validating, so a line break can survive.
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
$x = $_GET['x'];
if (filter_var($x, FILTER_VALIDATE_INT) !== false) {
    header('X-Id: ' . $x);
//  ^^^^^^^^^^^^^^^^^^^^^ TaintedHeader: Tainted HTTP header — possible header injection or open redirect
}
