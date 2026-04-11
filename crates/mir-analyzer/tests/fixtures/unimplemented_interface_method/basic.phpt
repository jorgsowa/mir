===source===
<?php
interface Runnable {
    public function run(): void;
}
class Task implements Runnable {}
===expect===
UnimplementedInterfaceMethod: class Task implements Runnable {}
