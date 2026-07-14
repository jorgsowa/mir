===description===
method_exists()/property_exists() true branch keeps the specific object class
and string atoms instead of collapsing everything to bare object.
===config===
suppress=UnusedVariable,UnusedParam,MixedMethodCall,MixedArgument
===file===
<?php
class Foo {
    public function bar(): void {}
}

function test_method_exists_keeps_class(Foo $f): void {
    if (method_exists($f, 'bar')) {
        /** @mir-check $f is Foo */
        $_ = $f;
    }
}

function test_property_exists_keeps_class(Foo $f): void {
    if (property_exists($f, 'x')) {
        /** @mir-check $f is Foo */
        $_ = $f;
    }
}

/** @param Foo|class-string<Foo> $x */
function test_method_exists_keeps_class_string(mixed $x): void {
    if (method_exists($x, 'bar')) {
        /** @mir-check $x is Foo|class-string<Foo> */
        $_ = $x;
    }
}

function test_method_exists_mixed_narrows_to_object(mixed $x): void {
    if (method_exists($x, 'bar')) {
        /** @mir-check $x is object */
        $_ = $x;
    }
}
===expect===
