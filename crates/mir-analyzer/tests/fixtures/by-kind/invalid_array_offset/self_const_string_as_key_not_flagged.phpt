===description===
self::STRING_CONST used as array key does not emit InvalidArrayOffset.
String constants are valid array keys; the literal string type is returned.
===config===
suppress=UnusedVariable
===file===
<?php
class Routes {
    const HOME    = 'home';
    const ABOUT   = 'about';
    const CONTACT = 'contact';

    /** @var array<string, callable> */
    private array $handlers = [];

    public function register(callable $fn): void {
        $this->handlers[self::HOME]    = $fn;
        $this->handlers[self::ABOUT]   = $fn;
        $this->handlers[self::CONTACT] = $fn;
    }
}
===expect===
