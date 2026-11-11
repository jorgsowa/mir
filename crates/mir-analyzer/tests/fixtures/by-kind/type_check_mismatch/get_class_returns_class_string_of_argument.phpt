===description===
`get_class($obj)` is `class-string<T>` for the object's class `T`, so a static property read through the result is typed.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
declare(strict_types=1);

final class Holder {
    public static int $count = 1;
}

function takesInt(int $value): void {}
function takesString(string $value): void {}

function fromGetClass(Holder $holder): void {
    $class = get_class($holder);
    /** @mir-check $class is class-string<Holder> */
    takesInt($class::$count);
    takesString($class::$count);
    //          ^^^^^^^^^^^^^^ InvalidArgument: Argument $value of takesString() expects 'string', got 'int'
    takesString($class);
}
