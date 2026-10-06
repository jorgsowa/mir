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
//<^^^^^^^^^^^^^^ InvalidTraitUse: Trait TraitB used incorrectly: TraitB has a circular trait composition chain
    use TraitA;
}
class Task implements Runnable {
//<^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ UnimplementedInterfaceMethod: Class Task must implement Runnable::run() from interface
    use TraitA;
}
