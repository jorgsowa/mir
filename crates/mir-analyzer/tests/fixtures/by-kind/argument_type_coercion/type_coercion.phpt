===description===
Type coercion
===file===
<?php
class A {}
class B extends A{}

function fooFoo(B $b): void {}
fooFoo(new A());
===expect===
ArgumentTypeCoercion@6:8-6:15: Argument $b of fooFoo() expects 'B', got 'A' — coercion may fail at runtime
