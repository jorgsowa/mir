===description===
A method declared `: self` returns the declaring class, so a subclass-only call on the result is reported against the declaring class.
===config===
suppress=UnusedMethod
===file===
<?php
class Base {
    public function returnsSelf(): self { return $this; }
    public static function make(): self { return new self(); }
}

class Sub extends Base {
    public function subOnly(): void {}
}

(new Sub())->returnsSelf()->nope();
(new Sub())->returnsSelf()->subOnly();
Sub::make()->subOnly();
===expect===
UndefinedMethod@11:0-11:34: Method Base::nope() does not exist
UndefinedMethod@12:0-12:37: Method Base::subOnly() does not exist
UndefinedMethod@13:0-13:22: Method Base::subOnly() does not exist
