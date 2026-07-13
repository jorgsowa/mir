===description===
FirstClassCallable:MethodExistsGuardSuppresses
===config===
suppress=MixedAssignment,UnusedVariable
===file===
<?php
class Widget {}
$w = new Widget();
if (method_exists($w, 'maybe')) {
    $closure = $w->maybe(...);
}
===expect===
