===description===
Prevent trait property type
===ignore===
TODO
===file===
<?php
trait T {}

class X {
  /** @var T|null */
  public $hm;
}
===expect===
