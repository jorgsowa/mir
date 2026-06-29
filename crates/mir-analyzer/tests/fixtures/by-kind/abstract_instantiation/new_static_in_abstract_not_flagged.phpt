===description===
new static() inside an abstract class does NOT fire — late static binding resolves to the concrete subclass at runtime.
===file===
<?php
abstract class Factory {
    public static function create(): static {
        return new static();
    }
}
===expect===
