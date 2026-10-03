===description===
Negative control: when no union atom accepts the arg count, every rejecting atom is still flagged
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class A {
    public function run(): void {}
}
class B {
    public function run(int $a = 0): void {}
}
function f(A|B $x): void {
    $x->run(1, 2);
//          ^ TooManyArguments: Too many arguments for run(): expected 0, got 2
//             ^ TooManyArguments: Too many arguments for run(): expected 1, got 2
}
===expect===
