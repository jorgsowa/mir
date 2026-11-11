===description===
A named function with visible side effects, passed as a first-class callable or by name, is not a `pure-callable`.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function announce(string $message): int {
    echo $message;
    return strlen($message);
}

function countCalls(): int {
    static $calls = 0;
    return ++$calls;
}

/** @impure */
function readClock(): int {
    return time();
}

function square(int $n): int {
    return $n * $n;
}

/** @param pure-callable $fn */
function memoize($fn): void {}

memoize(square(...));
memoize('square');
memoize(announce(...));
//      ^^^^^^^^^^^^^ InvalidArgument: Argument $fn of memoize() expects 'pure-callable', got 'impure Closure(string): int'
memoize('announce');
//      ^^^^^^^^^^ InvalidArgument: Argument $fn of memoize() expects 'pure-callable', got 'impure "announce"'
memoize(countCalls(...));
//      ^^^^^^^^^^^^^^^ InvalidArgument: Argument $fn of memoize() expects 'pure-callable', got 'impure Closure(): int'
memoize(readClock(...));
//      ^^^^^^^^^^^^^^ InvalidArgument: Argument $fn of memoize() expects 'pure-callable', got 'impure Closure(): int'
