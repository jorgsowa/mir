===description===
FirstClassCallable:MethodExistsGuardSuppresses
===config===
<mir>
  <issueHandlers>
    <MixedAssignment errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Widget {}
$w = new Widget();
if (method_exists($w, 'maybe')) {
    $closure = $w->maybe(...);
}
===expect===
