===description===
Narrowing an interface-typed value by an instanceof check for one of its
own implementors replaces (subsumes) rather than forming a redundant
Interface&Impl intersection
===config===
suppress=UnusedParam
===file===
<?php
interface Foo {}
class Impl implements Foo {}

function f(Foo $x): void {
    if ($x instanceof Impl) {
        /** @mir-check $x is Impl */
        echo get_class($x);
    }
}
===expect===
