===description===
A union receiver where a sibling atom's method accepts the arg count does not flag the atom that rejects it
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Plain {
    public function run(): void {}
}
class Wide {
    public function run(int $a = 0): void {}
}
class Variadic {
    public function run(int ...$a): void {}
}
function f(Plain|Wide $x, Plain|Variadic $y): void {
    $x->run(1);
    $y->run(1, 2);
}
