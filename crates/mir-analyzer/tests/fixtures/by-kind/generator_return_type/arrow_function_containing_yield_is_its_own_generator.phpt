===description===
`fn() => yield` is a generator; its yield does not make the enclosing function one
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function outer(): int {
    $g = fn() => yield 1;
    return 1;
}

function check(): void {
    $x = outer();
    /** @mir-check $x is int */
    $g = fn() => yield 'a';
    $y = $g();
    /** @mir-check $y is Generator<int, "a", mixed, mixed> */
    $h = fn(): \Generator => yield 1;
    $_ = 1;
}
