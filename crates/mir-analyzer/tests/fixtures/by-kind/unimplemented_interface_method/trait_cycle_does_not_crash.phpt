===description===
trait cycle does not crash
===file===
<?php
interface Runnable {
    public function run(): void;
}
// Mutual trait use — cycle guard must prevent infinite recursion.
// Neither trait provides run(), so the issue should still be reported.
trait TraitA {
    use TraitB;
}
trait TraitB {
    use TraitA;
}
class Task implements Runnable {
    use TraitA;
}
===expect===
InvalidTraitUse@10:0-10:14: Trait TraitB used incorrectly: TraitB has a circular trait composition chain
UnimplementedInterfaceMethod@13:0-13:32: Class Task must implement Runnable::run() from interface
