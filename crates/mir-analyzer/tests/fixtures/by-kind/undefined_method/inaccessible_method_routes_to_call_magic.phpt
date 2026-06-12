===description===
Calling a protected/private method from outside dispatches to __call when the
class defines one (Laravel Router::prefix via Macroable) — not UndefinedMethod.
Without __call the inaccessible call still errors.
===file===
<?php
class WithMagic {
    protected function prefix(string $uri): string { return $uri; }
    private function secret(): void {}
    public function __call($method, $parameters) { return null; }
}

class NoMagic {
    protected function prefix(string $uri): string { return $uri; }
}

$w = new WithMagic();
$w->prefix('foo');
$w->secret();

$n = new NoMagic();
$n->prefix('foo');
===expect===
UndefinedMethod@17:1-17:18: Method NoMagic::prefix() does not exist
