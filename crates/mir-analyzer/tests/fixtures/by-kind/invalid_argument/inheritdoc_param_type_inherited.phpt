===description===
@inheritdoc should cause a method to inherit its parent's @param types so that
passing the wrong argument type is still caught at the call site.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
  <phpVersion>8.2</phpVersion>
</mir>
===file===
<?php
class Cat {}

abstract class Handler {
    /** @param Cat $input */
    abstract public function handle(mixed $input): void;
}

class ConcreteHandler extends Handler {
    /** @inheritdoc */
    public function handle(mixed $input): void {}
}

function bad(ConcreteHandler $h): void {
    $h->handle("not a cat");
//             ^^^^^^^^^^^ InvalidArgument: Argument $input of handle() expects 'Cat', got '"not a cat"'
}
===expect===
