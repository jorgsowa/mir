===description===
Constructing a generic stub class binds its templates from the constructor
argument, the same way a user class does, so the instance satisfies a typed
Traversable/Iterator parameter and a mismatched one is still caught.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @param Traversable<string, int> $t */
function takesStringKeyed(Traversable $t): void {}

/** @param Iterator<string, int> $t */
function takesIterator(Iterator $t): void {}

function keyed(): void {
    $it = new ArrayIterator(['a' => 1, 'b' => 2]);
    /** @mir-check $it is ArrayIterator<string, int> */
    takesStringKeyed($it);
    takesIterator($it);
    takesStringKeyed(new ArrayIterator(['a' => 1]));
}

function arrayObject(): void {
    $ao = new ArrayObject(['a' => 1]);
    /** @mir-check $ao is ArrayObject<mixed, int> */
    takesStringKeyed($ao->getIterator());
}

function emptyConstructor(): void {
    $it = new ArrayIterator();
    /** @mir-check $it is ArrayIterator */
    takesStringKeyed($it);
}

function listKeysMismatch(): void {
    takesStringKeyed(new ArrayIterator([1, 2]));
}

function valueMismatch(): void {
    takesStringKeyed(new ArrayIterator(['a' => 'x']));
}
===expect===
InvalidArgument@29:21-29:46: Argument $t of takesStringKeyed() expects 'Traversable<string, int>', got 'ArrayIterator<int, int>'
InvalidArgument@33:21-33:52: Argument $t of takesStringKeyed() expects 'Traversable<string, int>', got 'ArrayIterator<string, string>'
