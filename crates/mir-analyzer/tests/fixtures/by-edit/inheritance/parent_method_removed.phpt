===description===
Removing a parent method reports calls through the unchanged child.
===file:Base.php===
<?php
class Base { public function hello(): void {} }
===file:Child.php===
<?php
class Child extends Base {}
===file:Use.php===
<?php
function run(Child $c): void { $c->hello(); }
===expect===
===edit:Base.php===
<?php
class Base {}
===expect===
Use.php: UndefinedMethod@2:31-2:42: Method Child::hello() does not exist
