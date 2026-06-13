===description===
Property docblock on property
===file===
<?php
class A {
   /** @property string[] */
  public array $arr;
}
===expect===
MissingConstructor@2:0-2:9: Class A has uninitialized properties but no constructor
