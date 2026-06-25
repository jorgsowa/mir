===description===
InvalidStaticInvocation does NOT fire when the class defines __callStatic.
===file===
<?php
class Proxy {
    public function forward(): void {}
    public static function __callStatic(string $method, array $args) {}
}

Proxy::forward();
===expect===
