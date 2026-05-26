===description===
Wrong case class with call
===file===
<?php
class A {}
needsA(new A);
function needsA(a $x): void {}
===expect===
InvalidClass
===ignore===
TODO
