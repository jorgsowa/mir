===description===
case insensitive method not reported
===file===
<?php
interface HasFooBar {
    public function fooBar(): void;
}
class Impl implements HasFooBar {
    public function fooBar(): void {}
}
===expect===
