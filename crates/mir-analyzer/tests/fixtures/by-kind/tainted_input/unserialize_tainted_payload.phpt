===description===
tainted payload reaching unserialize is reported (object injection)
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function test(): void {
    $obj = unserialize($_COOKIE['session']);
//         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedInput: Tainted input reaching sink 'unserialize'
}
===expect===
