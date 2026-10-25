===description===
Deprecated class with new attr
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
#[\Deprecated]
//^^^^^^^^^^^ CompileError: Cannot apply #[\Deprecated] to class Foo [deprecated-on-invalid-target]
class Foo { }

$a = new Foo();
//       ^^^ DeprecatedClass: Class Foo is deprecated
