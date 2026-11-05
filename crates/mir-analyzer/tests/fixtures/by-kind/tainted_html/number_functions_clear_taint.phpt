===description===
`abs`, `round`, `floor` and `ceil` return numbers whatever they receive.
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
echo abs($_GET['a']), round($_GET['b']), floor($_GET['c']), ceil($_GET['d']);
