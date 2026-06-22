===description===
PHP class lookup is ASCII-case-insensitive only. Ñoño and ñoño differ in
non-ASCII bytes (Ñ ≠ ñ), so the exact name is not found; the result is
UndefinedClass, not WrongCaseClass. The exact-match spelling is not reported.
===config===
suppress=UnusedVariable
===file===
<?php
class Ñoño {}
$a = new Ñoño();
$b = new ñoño();
===expect===
UndefinedClass@4:9-4:13: Class ñoño does not exist
