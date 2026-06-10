===description===
Array type coercion
===ignore===
TODO
===file===
<?php
class A {}
class B extends A{}

/**
 * @param  B[]  $b
 * @return void
 */
function fooFoo(array $b) {}
fooFoo([new A()]);
===expect===
