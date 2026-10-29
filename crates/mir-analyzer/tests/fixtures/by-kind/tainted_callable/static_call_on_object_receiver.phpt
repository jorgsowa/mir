===description===
`$obj::m()` on an object dispatches on the object's class, so tainted data in the object does not choose the code; a tainted class-name string still does.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
    <MixedMethodCall errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
final class R {
    public static function m(): void {}
}

function f(): void {
    $r = unserialize($_GET['x'], ['allowed_classes' => [R::class]]);
//       ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedInput: Tainted input reaching sink 'unserialize'
    if ($r instanceof R) {
        $r::m();
    }
    $name = $_GET['c'];
    $name::m();
//  ^^^^^^^^^^ TaintedCallable: Tainted callable — user input chooses the code that runs
}
