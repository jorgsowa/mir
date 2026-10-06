===description===
`new $x()` where $x is typed interface-string must error: an interface name can
never be instantiated, unlike class-string<AbstractClass> which may still hold a
concrete non-abstract subclass name at runtime.
===config===
<mir>
  <issueHandlers>
    <MissingReturnType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
interface Shape {}

function test(string $className) {
    /** @var interface-string<Shape> $className */
    new $className();
//      ^^^^^^^^^^ InvalidStringClass: Dynamic class instantiation requires string or class-string type, got 'interface-string<Shape>'
}
