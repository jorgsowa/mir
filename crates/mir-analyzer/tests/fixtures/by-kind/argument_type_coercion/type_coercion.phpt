===description===
Type coercion
===config===
suppress=UnusedParam
===file===
<?php
class A {}
class B extends A{}

function fooFoo(B $b): void {}
fooFoo(new A());
===expect===
ArgumentTypeCoercion@6:7-6:14: Argument $b of fooFoo() expects 'B', got 'A' — coercion may fail at runtime
