===description===
A closure whose body echoes, prints or exits is not a `pure-callable`; one that only computes is.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @param pure-callable $transform */
function apply($transform): void {}

$double = static fn (int $n): int => $n * 2;
$logging = static function (int $n): int {
    echo $n;
    return $n;
};
$printing = static fn (int $n): int => print $n;
$stopping = function (): void {
    exit(1);
};

apply($double);
apply($logging);
//    ^^^^^^^^ InvalidArgument: Argument $transform of apply() expects 'pure-callable', got 'impure Closure(int): int'
apply($printing);
//    ^^^^^^^^^ InvalidArgument: Argument $transform of apply() expects 'pure-callable', got 'impure Closure(int): positive-int'
apply($stopping);
//    ^^^^^^^^^ InvalidArgument: Argument $transform of apply() expects 'pure-callable', got 'impure Closure(): void'
apply(static function (): void {
//    ^ +2:1 InvalidArgument: Argument $transform of apply() expects 'pure-callable', got 'impure Closure(): void'
    echo 'inline';
});
