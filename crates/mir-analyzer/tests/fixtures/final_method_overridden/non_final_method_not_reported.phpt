===description===
non final method not reported
===file===
<?php
class ParentClass {
    public function unlocked(): void {}
}
class Child extends ParentClass {
    public function unlocked(): void {}
}
===expect===
===ignore===
TODO
