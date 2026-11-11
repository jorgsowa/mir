===description===
A scalar passed for a class-typed shape value is `InvalidArgument`, from a literal or from a declared shape. An object of the class, or one whose class is not known statically, is still accepted.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
final class Foo {}

/** @param array{item: Foo} $box */
function takesBox(array $box): void {}

takesBox(['item' => new Foo()]);
takesBox(['item' => 'nope']);
//       ^^^^^^^^^^^^^^^^^^ InvalidArgument: Argument $box of takesBox() expects 'array{'item': Foo}', got 'array{'item': "nope"}'

/** @param array{item: int} $ints */
function passesInts(array $ints, object $any): void {
    takesBox($ints);
//           ^^^^^ InvalidArgument: Argument $box of takesBox() expects 'array{'item': Foo}', got 'array{'item': int}'
    takesBox(['item' => $any]);
}
