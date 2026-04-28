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
UndefinedMethod: Method Widget::undefinedMethod() does not exist
