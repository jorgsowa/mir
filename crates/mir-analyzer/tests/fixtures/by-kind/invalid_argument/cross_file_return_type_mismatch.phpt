===description===
Cross-file return type mismatch produces InvalidArgument
===config===
suppress=UnusedParam
===file:Maker.php===
<?php
class Apple {}
class Banana {}
class Maker {
    public function make(): Apple { return new Apple(); }
}
===file:Consumer.php===
<?php
function expect_banana(Banana $v): void {}
expect_banana((new Maker)->make());
===expect===
Consumer.php: InvalidArgument@3:14-3:33: Argument $v of expect_banana() expects 'Banana', got 'Apple'
