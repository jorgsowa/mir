===description===
Type coercion
===file===
<?php
class A {}
class B extends A{}

function fooFoo(B $b): void {}
fooFoo(new A());
===expect===
ArgumentTypeCoercion
===ignore===
TODO
