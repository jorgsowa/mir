===description===
`@if-this-is` must be checked on `self::`/`static::`/`parent::` call
syntax too, not just `$this->method()` — analyze_static_method_call never
invoked the check at all.
===config===
suppress=MissingPropertyType,UnusedParam
===file===
<?php
trait Foo {
    /** @if-this-is Bar */
    public function onlyBar(): void {}
}

class Bar {
    use Foo;

    public function ok(): void {
        self::onlyBar();
    }
}

class Baz {
    use Foo;

    public function callIt(): void {
        self::onlyBar();
        static::onlyBar();
    }
}

/** @template T */
class Box {
    /** @var T */ private $v;
    /** @param T $v */
    public function __construct($v) { $this->v = $v; }
    /** @if-this-is Box<int> */
    public function onlyInt(): void {}
    public function relay(): void {
        // $this carries no concrete type args inside its own generic class
        // body, so this must stay unflagged (same as the $this-> syntax).
        self::onlyInt();
        static::onlyInt();
    }
}
===expect===
IfThisIsMismatch@19:8-19:23: Cannot call Baz::onlyBar() — @if-this-is requires $this to be 'Bar', but it is 'Baz'
IfThisIsMismatch@20:8-20:25: Cannot call Baz::onlyBar() — @if-this-is requires $this to be 'Bar', but it is 'Baz'
