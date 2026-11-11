===description===
`stringable-object` is enforced as a union member and as a constructor parameter type.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
final class Version {
    public function __toString(): string {
        return '1.0';
    }
}

final class Build {
    public int $number = 7;
}

/** @param stringable-object|int $value */
function show($value): void {}

final class Banner {
    /** @param stringable-object $text */
    public function __construct(public object $text) {}
}

show(new Version());
show(3);
show(new Build());
//   ^^^^^^^^^^^ InvalidArgument: Argument $value of show() expects 'Stringable|int', got 'Build'
show(1.5);
//   ^^^ InvalidArgument: Argument $value of show() expects 'Stringable|int', got '1.5'
new Banner(new Version());
new Banner(new Build());
//         ^^^^^^^^^^^ InvalidArgument: Argument $text of Banner::__construct() expects 'Stringable', got 'Build'
