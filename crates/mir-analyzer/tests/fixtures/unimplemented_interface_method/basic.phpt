===description===
basic
===file===
<?php
interface Runnable {
    public function run(): void;
}
class Task implements Runnable {}
===expect===
UnimplementedInterfaceMethod@5:0: Class Task must implement Runnable::run() from interface
