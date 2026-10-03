===description===
Mixed property fetch
===config===
<mir>
  <issueHandlers>
    <MissingPropertyType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Foo {
    /** @var string */
    public $foo = "";
}

/** @var mixed */
$a = (new Foo());

echo $a->foo;
//   ^^^^^^^ MixedPropertyFetch: Property $foo fetched on mixed type
===expect===
