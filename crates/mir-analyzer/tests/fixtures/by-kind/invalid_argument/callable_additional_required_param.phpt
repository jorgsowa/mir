===description===
Callable additional required param
===config===
<mir>
  <issueHandlers>
    <ForbiddenCode errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/**
 * @param callable(string, string, string):bool $arg
 * @return void
 */
function foo($arg) {
    var_dump($arg);
}

function bar(string $a, string $b, string $c, string $d): bool {
    return $a . $b . $c . $d ? true : false;
}

foo("bar");
//  ^^^^^ InvalidArgument: Argument $arg of foo() expects 'callable with 3 required parameter(s)', got 'callable with 4 required parameter(s)'
