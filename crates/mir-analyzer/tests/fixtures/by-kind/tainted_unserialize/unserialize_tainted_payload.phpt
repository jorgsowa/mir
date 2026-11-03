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
//         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedUnserialize: Tainted unserialize() payload — possible object injection
}
