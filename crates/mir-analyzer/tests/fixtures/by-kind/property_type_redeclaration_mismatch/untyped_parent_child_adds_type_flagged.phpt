===description===
Child adds a native type hint where the parent had none: PHP requires it omitted (fatal on every version).
===config===
<mir>
  <issueHandlers>
    <MissingPropertyType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class A {
    public $x = 1;
}

class B extends A {
    public int $x = 2;
//  ^^^^^^^^^^^^^^^^^^ PropertyTypeRedeclarationMismatch: Type of B::$x must be omitted to match the parent definition in class A
}
