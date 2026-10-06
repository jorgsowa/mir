===description===
Undefined callable method array
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class A {
    public static function bar(string $a): string {
        return $a . "b";
    }
}

function foo(callable $c): void {}

foo([A::class, "::barr"]);
//  ^^^^^^^^^^^^^^^^^^^^ UndefinedMethod: Method A::::barr() does not exist
