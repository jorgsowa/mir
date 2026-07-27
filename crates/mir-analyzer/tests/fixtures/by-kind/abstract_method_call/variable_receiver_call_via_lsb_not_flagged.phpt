===description===
AbstractMethodCall does NOT fire for $var::method() when $var is typed as the abstract class itself — the variable can only ever hold a concrete subclass instance at runtime, since the abstract class can never be instantiated.
===file===
<?php
abstract class Loader {
    abstract public static function getType(): string;
}
class Consumer {
    public function run(Loader $l): string {
        return $l::getType();
    }
}
===expect===
