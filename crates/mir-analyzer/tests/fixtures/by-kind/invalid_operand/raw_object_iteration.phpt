===description===
Raw object iteration
===config===
<mir>
  <issueHandlers>
    <MissingPropertyType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class A {
    /** @var ?string */
    public $foo;
}
function example() : Generator {
    $arr = new A;

    yield from $arr;
//             ^^^^ RawObjectIteration: Cannot iterate over non-iterable object 'A'
}
