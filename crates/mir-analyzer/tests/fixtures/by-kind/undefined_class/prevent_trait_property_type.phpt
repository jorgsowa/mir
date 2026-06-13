===description===
Prevent trait property type
===config===
suppress=MissingPropertyType
===file===
<?php
trait T {}

class X {
  /** @var T|null */
  public $hm;
}
===expect===
