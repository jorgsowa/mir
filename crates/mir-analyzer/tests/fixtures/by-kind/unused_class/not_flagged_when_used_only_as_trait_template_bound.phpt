===description===
A class named only as a trait's own `@template T of Bound` bound must not be reported UnusedClass.
===config===
suppress=
===file===
<?php
final class Bound {
}

/** @template T of Bound */
trait MyTrait {
}

final class Impl {
    use MyTrait;
}

new Impl();
===expect===
