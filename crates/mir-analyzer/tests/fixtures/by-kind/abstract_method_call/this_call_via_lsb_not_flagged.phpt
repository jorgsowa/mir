===description===
AbstractMethodCall does NOT fire for $this::method() on an abstract method — $this uses LSB and resolves to the concrete runtime class, which must implement the method (an abstract class can never be instantiated).
===file===
<?php
abstract class Loader {
    abstract public static function getType(): string;
    public function describe(): string {
        return $this::getType();
    }
}
class Concrete extends Loader {
    public static function getType(): string { return "concrete"; }
}
===expect===
