===description===
`A::f()` → `B::g()` → `A::h()` re-enters `A`'s scope while it is still being
inferred. The fixpoint resolves the cycle, so `B::g()` infers `1` from
`A::h()` instead of degrading to `mixed`.
===config===
<mir>
  <issueHandlers>
    <MissingReturnType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file:A.php===
<?php
class A {
    function f() { return B::g(); }
    function h() { return 1; }
}
===file:B.php===
<?php
class B {
    static function g() { return (new A)->h(); }
}
===file:Z.php===
<?php
function test(): string { return B::g(); }
//                        ^^^^^^^^^^^^^^ InvalidReturnType: Return type '1' is not compatible with declared 'string'
===expect===
