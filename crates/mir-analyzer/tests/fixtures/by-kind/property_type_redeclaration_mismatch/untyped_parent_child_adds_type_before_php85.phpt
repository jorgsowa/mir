===description===
Before PHP 8.5 a type added where the parent had none reads "must not be defined".
===config===
<mir>
  <phpVersion>8.4</phpVersion>
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
//  ^^^^^^^^^^^^^^^^^^ PropertyTypeRedeclarationMismatch: Type of B::$x must not be defined (as in class A)
}
