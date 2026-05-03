===description===
closure return type resolved
===file===
<?php
/**
 * @template T
 */
class Factory {
    /** @return \Closure(): T */
    public function maker(): \Closure { return function() { return null; }; }
}
class Widget { public function render(): void {} }
function test(): void {
    /** @var Factory<Widget> $f */
    $f = new Factory();
    $maker = $f->maker();
    $maker()->undefinedMethod();
}
===expect===
UndefinedMethod@14:4: Method Widget::undefinedMethod() does not exist
===ignore===
TODO
