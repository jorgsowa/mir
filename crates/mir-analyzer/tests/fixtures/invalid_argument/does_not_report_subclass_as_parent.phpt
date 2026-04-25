===file===
<?php
class Base {}
class Child extends Base {}
function f(Base $x): void { var_dump($x); }
function test(): void { f(new Child()); }
===expect===
