===description===
method_exists()/property_exists() false branch also excludes non-object/
non-string atoms — PHP throws TypeError for those regardless of which
boolean the call returns, so reaching the false branch proves it too.
===config===
suppress=UnusedVariable,UnusedParam,PossiblyInvalidArgument
===file===
<?php
class Foo { public function bar(): void {} }

/** @param int|Foo $x */
function test_method_exists_false_branch($x): void {
    if (!method_exists($x, 'bar')) {
        /** @mir-check $x is Foo */
        $_ = $x;
    }
}

/** @param int|Foo $x */
function test_property_exists_false_branch($x): void {
    if (!property_exists($x, 'bar')) {
        /** @mir-check $x is Foo */
        $_ = $x;
    }
}
===expect===
