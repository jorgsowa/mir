===description===
P3: First-class callable on an unknown method reports UndefinedMethod (like
the ordinary call form) instead of silently falling back to untyped callable.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php

class MyClass {}

$obj = new MyClass();
$fn = $obj->undefined(...);
//          ^^^^^^^^^ UndefinedMethod: Method MyClass::undefined() does not exist
