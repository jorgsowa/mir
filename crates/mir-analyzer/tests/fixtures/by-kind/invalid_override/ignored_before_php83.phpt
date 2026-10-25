===description===
Before PHP 8.3 `Override` is not a built-in attribute and the method is not checked.
===config===
<mir>
  <phpVersion>8.2</phpVersion>
</mir>
===file===
<?php
final class C {
    #[Override]
//    ^^^^^^^^ UndefinedAttributeClass: Attribute class Override does not exist
    public function m(): void {}
}
