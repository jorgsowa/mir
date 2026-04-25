===file===
<?php
class Foo {
    public function __toString(): string {
        return '';
    }

    public function __invoke(int $x): void {}

    public function __debugInfo(): array {
        return [];
    }
}
===expect===
UnusedParam: Parameter $x is never used
