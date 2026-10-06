===description===
non runtime magic methods reported
===file===
<?php
class Foo {
    public function __toString(): string {
        return '';
    }

    public function __invoke(int $x): void {}
//                           ^^^^^^ UnusedParam: Parameter $x is never used

    public function __debugInfo(): array {
        return [];
    }
}
