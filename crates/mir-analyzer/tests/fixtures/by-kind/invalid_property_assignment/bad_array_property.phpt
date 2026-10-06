===description===
Bad array property
===config===
<mir>
  <issueHandlers>
    <MissingPropertyType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class A {}

class B {}

class C {
    /** @var array<B> */
    public $bb;
}

$c = new C;
$c->bb = [new A, new B];
