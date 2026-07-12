===description===
A class named only as an interface's own `@template T of Bound` bound must not be reported UnusedClass.
===config===
suppress=
===file===
<?php
final class Bound {
}

/** @template T of Bound */
interface MyIface {
}

final class Impl implements MyIface {
}

new Impl();
===expect===
