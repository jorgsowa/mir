===description===
Each literal of a ternary argument is checked against `class-string`, for methods as for functions.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
final class Report {}

final class Registry {
    /** @param class-string $class */
    public function add(string $class): void {}

    /** @param class-string $class */
    public static function has(string $class): bool {
        return $class !== '';
    }
}

function run(Registry $registry, bool $flag): void {
    $registry->add($flag ? Report::class : 'no such thing');
//                 ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InvalidArgument: Argument $class of add() expects 'class-string', got '"no such thing"'
    $registry->add($flag ? Report::class : 'Report');
    Registry::has('a.b.c');
//                ^^^^^^^ InvalidArgument: Argument $class of has() expects 'class-string', got '"a.b.c"'
    Registry::has('Report');
}
