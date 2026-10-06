===description===
@readonly docblock (advisory, not runtime-enforced) dropped in child — no error
===config===
<mir>
  <issueHandlers>
    <MissingPropertyType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class A {
    /** @readonly */
    public $x = 0;
}

class B extends A {
    public $x = 1;
}
