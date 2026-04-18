===source===
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
UnimplementedInterfaceMethod: class Task implements Runnable {
