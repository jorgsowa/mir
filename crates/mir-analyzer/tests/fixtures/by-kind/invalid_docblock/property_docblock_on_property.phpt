===description===
Property docblock on property
===file===
<?php
class A {
//<^^^^^^^^^ MissingConstructor: Class A has uninitialized properties but no constructor
   /** @property string[] */
  public array $arr;
}
