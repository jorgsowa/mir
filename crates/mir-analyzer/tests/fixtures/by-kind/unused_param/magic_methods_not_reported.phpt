===description===
magic methods not reported
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
CompileError@13:4-15:5: Compile error: Method Magic::__callStatic() must be static [magic-method-static]
