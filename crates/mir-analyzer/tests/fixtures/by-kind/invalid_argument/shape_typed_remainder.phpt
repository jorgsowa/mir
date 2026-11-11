===description===
A shape with a typed remainder (`array{name: string, ...<Foo>}`) accepts extra keys only when their values fit the remainder type; the listed keys stay required and typed.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
final class Foo {}

/** @param array{name: string, ...<Foo>} $row */
function takesRow(array $row): void {}

/** @param array{name: string, ...<string, int>} $counts */
function takesCounts(array $counts): void {}

takesRow(['name' => 'Ada']);
takesRow(['name' => 'Ada', 'extra' => new Foo()]);
takesRow(['name' => 'Ada', 'extra' => 'nope']);
//       ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InvalidArgument: Argument $row of takesRow() expects 'array{'name': string, ...<Foo>}', got 'array{'name': "Ada", 'extra': "nope"}'
takesRow(['extra' => new Foo()]);
//       ^^^^^^^^^^^^^^^^^^^^^^ InvalidArgument: Argument $row of takesRow() expects 'array{'name': string, ...<Foo>}', got 'array{'extra': Foo}'

takesCounts(['name' => 'Ada', 'a' => 1, 'b' => 2]);
takesCounts(['name' => 'Ada', 'a' => 1.5]);
//          ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InvalidArgument: Argument $counts of takesCounts() expects 'array{'name': string, ...<int>}', got 'array{'name': "Ada", 'a': 1.5}'

/** @param array{name: string, extra: string} $wider */
function passesWider(array $wider): void {
    takesRow($wider);
//           ^^^^^^ InvalidArgument: Argument $row of takesRow() expects 'array{'name': string, ...<Foo>}', got 'array{'name': string, 'extra': string}'
}
