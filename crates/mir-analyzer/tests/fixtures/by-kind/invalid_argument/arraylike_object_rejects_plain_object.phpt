===description===
`arraylike-object` rejects objects without array-like capabilities.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
final class PlainObject {}

/** @param arraylike-object<string, int> $bag */
function takesArraylike($bag): void {}

takesArraylike(new PlainObject());
//             ^^^^^^^^^^^^^^^^^ InvalidArgument: Argument $bag of takesArraylike() expects 'ArrayAccess<string, int>&Countable&Traversable<string, int>', got 'PlainObject'
===expect===
