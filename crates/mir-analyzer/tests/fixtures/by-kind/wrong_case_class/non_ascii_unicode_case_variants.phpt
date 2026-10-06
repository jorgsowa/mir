===description===
PHP class lookup is ASCII-case-insensitive only. Ñoño and ñoño differ in
non-ASCII bytes (Ñ ≠ ñ), so the exact name is not found; the result is
UndefinedClass, not WrongCaseClass. The exact-match spelling is not reported.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Ñoño {}
$a = new Ñoño();
$b = new ñoño();
//       ^^^^ UndefinedClass: Class ñoño does not exist
===expect===
