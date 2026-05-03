===description===
implemented not reported
===file===
<?php
interface Runnable {
    public function run(): void;
}
class Task implements Runnable {
    public function run(): void {}
}
===expect===
===ignore===
TODO
