===description===
A class named only inside an `@extends`/`@implements` generic
type-argument list, or a `@template T of Bound`, must not be reported
UnusedClass.
===config===
suppress=
===file===
<?php
final class OnlyUsedInExtendsTypeArg {}
final class OnlyUsedInTemplateBound {}

/** @template T */
class Box {}

/** @extends Box<OnlyUsedInExtendsTypeArg> */
class ConcreteBox extends Box {}

/** @template T of OnlyUsedInTemplateBound */
class BoundedBox {}
===expect===
