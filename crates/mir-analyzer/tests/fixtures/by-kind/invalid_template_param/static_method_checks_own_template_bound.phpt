===description===
A static method's own `@template U of Bound` was never enforced — only
instance-method and free-function calls checked template bounds. `Foo::bar()`
(and `self::`/`parent::` static calls) must check the bound too.
===config===
suppress=UnusedVariable,MissingPropertyType
===file===
<?php
class Base {}
class NotBase {}

/** @template T of Base */
class Box {
    /** @param T $item */
    private function __construct(private $item) {}

    /**
     * @template U of Base
     * @param U $item
     * @return Box<U>
     */
    public static function make($item): self {
        return new self($item);
    }
}

// Satisfies the bound — no error.
$ok = Box::make(new Base());

// Violates the bound — NotBase does not extend Base.
$bad = Box::make(new NotBase());
===expect===
InvalidTemplateParam@24:7-24:31: Template type 'U' inferred as 'NotBase' does not satisfy bound 'Base'
