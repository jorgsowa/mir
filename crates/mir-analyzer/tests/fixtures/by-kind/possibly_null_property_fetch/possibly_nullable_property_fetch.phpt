===description===
Possibly nullable property fetch
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

$a = rand(0, 10) ? new Foo() : null;

echo $a->foo;
//   ^^^^^^^ PossiblyNullPropertyFetch: Cannot access property $foo on possibly null value
===expect===
