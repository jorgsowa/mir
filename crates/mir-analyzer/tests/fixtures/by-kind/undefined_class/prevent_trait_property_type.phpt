===description===
Prevent trait property type
===config===
<mir>
  <issueHandlers>
    <MissingPropertyType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
trait T {}

class X {
  /** @var T|null */
  public $hm;
}
===expect===
