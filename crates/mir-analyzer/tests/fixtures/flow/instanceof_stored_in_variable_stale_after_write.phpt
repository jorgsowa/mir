===file===
<?php
class A {}
class B extends A { public function b(): void {} }

function reassigned(A $x): void {
    $is = $x instanceof B;
    $x = new A();
    if ($is) { $x->b(); }
//             ^^^^^^^ UndefinedMethod: Method A::b() does not exist
}

function reassigned_on_a_branch(A $x, bool $c): void {
    $is = $x instanceof B;
    if ($c) { $x = new A(); }
    if ($is) { $x->b(); }
//             ^^^^^^^ UndefinedMethod: Method A::b() does not exist
}

function flag_rewritten(A $x, bool $c): void {
    $is = $x instanceof B;
    if ($c) { $is = true; }
    if ($is) { $x->b(); }
//             ^^^^^^^ UndefinedMethod: Method A::b() does not exist
}

function by_ref_capture(A $x): void {
    $is = $x instanceof B;
    $f = function () use (&$x): void { $x = new A(); };
    $f();
    if ($is) { $x->b(); }
//             ^^^^^^^ UndefinedMethod: Method A::b() does not exist
}

function reference(A $x): void {
    $is = $x instanceof B;
    $r = &$x;
    $r = new A();
    if ($is) { $x->b(); }
//             ^^^^^^^ UndefinedMethod: Method A::b() does not exist
}
