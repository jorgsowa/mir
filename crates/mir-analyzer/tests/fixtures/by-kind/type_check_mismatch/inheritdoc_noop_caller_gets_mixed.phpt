===description===
Without @inheritdoc, a child method's return type at call sites remains what
the method itself declares (mixed here), not what the parent's docblock says.
The @mir-check below would fail if the child silently inherited Cat.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
    <MixedArgument errorLevel="suppress"/>
  </issueHandlers>
  <phpVersion>8.2</phpVersion>
</mir>
===file===
<?php
class Cat {}

abstract class Base {
    /** @return Cat */
    abstract public function make(): mixed;
}

class Child extends Base {
    public function make(): mixed {
        return new Cat();
    }
}

function test(Child $c): void {
    $result = $c->make();
    /** @mir-check $result is mixed */
    echo get_class($result);
}
===expect===
