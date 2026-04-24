===file:Base.php===
<?php
class Base {}
===file:Child.php===
<?php
class Child extends Base {}
===file:Consumer.php===
<?php
function accept(Base $x): void { var_dump($x); }
function test(): void {
    accept(new Child());
}
===expect===
