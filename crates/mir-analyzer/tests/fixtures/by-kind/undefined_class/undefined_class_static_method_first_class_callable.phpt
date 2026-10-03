===description===
A static-method first-class-callable (`Foo::bar(...)`) on an undefined class
must report UndefinedClass, matching the plain `Foo::bar()` call form.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
$c = MissingClass::baz(...);
//   ^^^^^^^^^^^^ UndefinedClass: Class MissingClass does not exist
===expect===
