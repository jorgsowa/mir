===description===
closure return type resolved
===config===
suppress=MissingClosureReturnType
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
UndefinedMethod@14:5-14:32: Method Widget::undefinedMethod() does not exist
