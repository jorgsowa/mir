===description===
PHP warns when a magic method it calls from outside the class is protected or private, in classes, traits and enums, abstract ones included.
===file===
<?php
class Config {
    protected function __get(string $name): mixed { return $name; }
//                     ^^^^^ NonPublicMagicMethod: The magic method Config::__get() must have public visibility
    private static function __callStatic(string $name, array $args): mixed { return $args; }
//                          ^^^^^^^^^^^^ NonPublicMagicMethod: The magic method Config::__callStatic() must have public visibility
}

trait Printable {
    private function __toString(): string { return ''; }
//                   ^^^^^^^^^^ NonPublicMagicMethod: The magic method Printable::__toString() must have public visibility
}

enum Action {
    case Run;

    protected function __invoke(): int { return 1; }
//                     ^^^^^^^^ NonPublicMagicMethod: The magic method Action::__invoke() must have public visibility
}

abstract class Lazy {
    abstract protected function __isset(string $name): bool;
//                              ^^^^^^^ NonPublicMagicMethod: The magic method Lazy::__isset() must have public visibility
}
