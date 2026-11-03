===description===
A class named only inside an `@extends`/`@implements` generic
type-argument list, or a `@template T of Bound`, must not be reported
UnusedClass.
===config===
<mir>
  <findUnusedCode>true</findUnusedCode>
</mir>
===file===
<?php
final class OnlyUsedInExtendsTypeArg {}
final class OnlyUsedInTemplateBound {}

/** @template T */
class Box {}

/** @extends Box<OnlyUsedInExtendsTypeArg> */
class ConcreteBox extends Box {}
//<^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ UnusedClass: Class ConcreteBox is never referenced

/** @template T of OnlyUsedInTemplateBound */
class BoundedBox {}
//<^^^^^^^^^^^^^^^^^^^ UnusedClass: Class BoundedBox is never referenced
