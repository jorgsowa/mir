===description===
Cross-file method return type propagates to variable assignment
===file:Maker.php===
<?php
class Apple {}
class Maker {
    public function make(): Apple { return new Apple(); }
}
===file:Consumer.php===
<?php
$x = (new Maker)->make();
/** @mir-check $x is Apple */
$_ = $x;
===expect===
