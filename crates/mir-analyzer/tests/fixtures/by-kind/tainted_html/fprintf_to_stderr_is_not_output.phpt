===description===
`fprintf` to `STDERR` is no HTML output; an unknown handle still counts.
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
fprintf(STDERR, "%s", $_GET['x']);
vfprintf(\STDERR, "%s", [$_GET['x']]);

function unknown($handle): void {
//               ^^^^^^^ MissingParamType: Parameter $handle of unknown() has no type annotation
    fprintf($handle, "%s", $_GET['x']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}
