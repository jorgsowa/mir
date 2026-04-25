===file===
<?php
class Magic {
    public function __get(string $name): mixed {
        return null;
    }

    public function __set(string $name, mixed $value): void {}

    public function __call(string $name, array $arguments): mixed {
        return null;
    }

    public function __callStatic(string $name, array $arguments): mixed {
        return null;
    }

    public function __isset(string $name): bool {
        return false;
    }

    public function __unset(string $name): void {}
}
===expect===
