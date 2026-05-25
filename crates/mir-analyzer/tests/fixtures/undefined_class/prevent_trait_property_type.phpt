===description===
Prevent trait property type
===file===
<?php
trait T {}

class X {
  /** @var T|null */
  public $hm;
}
===expect===
UndefinedDocblockClass
===ignore===
TODO
