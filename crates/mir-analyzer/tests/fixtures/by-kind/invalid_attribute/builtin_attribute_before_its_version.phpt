===description===
Before PHP 8.3 `Override` is not a built-in attribute, so a misplaced one is only an unknown attribute class.
===config===
<mir>
  <phpVersion>8.2</phpVersion>
</mir>
===file===
<?php
#[Override]
//^^^^^^^^ UndefinedAttributeClass: Attribute class Override does not exist
final class A {}
