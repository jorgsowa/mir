===description===
A class named only inside an interface's own `@extends` generic type-argument list must not be reported UnusedClass.
===config===
suppress=
===file===
<?php
final class Arg {
}

/** @template T */
interface Box {
}

/** @extends Box<Arg> */
interface IntBox extends Box {
}

final class Impl implements IntBox {
}

new Impl();
===expect===
