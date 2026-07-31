===description===
Negative control for the L30 fix: a receiver already typed as a concrete
class (not bare `object`) must keep its own class through `is_callable()`
narrowing, not get widened to a generic callable — only a bare `object`
lacks enough information to do better.
===config===
suppress=UnusedParam,MissingConstructor,UnusedVariable
===file===
<?php
class Handler {
    public function __invoke(): void {}
}

function test(Handler $h): void {
    if (is_callable($h)) {
        /** @mir-check $h is Handler */
        $x = $h;
    }
}
===expect===
