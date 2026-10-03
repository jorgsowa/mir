===description===
A union receiver where a sibling atom accepts fewer args does not flag TooFewArguments on the stricter atom
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Strict {
    public function run(int $a): void {}
}
class Lax {
    public function run(): void {}
}
function f(Strict|Lax $x): void {
    $x->run();
}
===expect===
