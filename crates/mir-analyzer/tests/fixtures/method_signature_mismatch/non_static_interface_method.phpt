===description===
Non static interface method
===file===
<?php
interface I {
    public static function m(): void;
}
class C implements I {
    public function m(): void {}
}
===expect===
MethodSignatureMismatch
===ignore===
TODO
