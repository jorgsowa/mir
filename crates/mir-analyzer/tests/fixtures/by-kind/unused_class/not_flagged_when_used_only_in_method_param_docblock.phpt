===description===
A class named only inside a method's `@param` docblock (no native type hint) must not be reported UnusedClass.
===config===
suppress=
===file===
<?php
final class Target {
}

final class Foo {
    /** @param Target $x */
    public function bar($x): void {
//                      ^^ UnusedParam: Parameter $x is never used
    }
}

new Foo();
===expect===
