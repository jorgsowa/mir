===description===
propertyDocblockOnProperty
===file===
<?php
class A {
   /** @property string[] */
  public array $arr;
}
===expect===
InvalidDocblock
===ignore===
TODO
