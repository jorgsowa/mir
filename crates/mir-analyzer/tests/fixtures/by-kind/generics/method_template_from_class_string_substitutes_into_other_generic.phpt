===description===
A method-level `@template T` bound from a `class-string<T>` argument must
substitute into a *different* generic class's return type through `$this->`,
mirroring the standalone-function proof in
`invalid_return_type/class_string_template_substitution.phpt` but for an
instance method call.
===config===
suppress=MissingReturnType,MissingParamType,UnusedParam,UnusedVariable
===file===
<?php
/** @template TValue */
class Box {
    /** @param TValue $v */
    public function __construct($v) {}
}

class Container {
    /**
     * @template TValue
     * @param class-string<TValue> $cls
     * @return Box<TValue>
     */
    public function make(string $cls): Box { return new Box(new $cls()); }
}

class Widget {}

$c = new Container();
$box = $c->make(Widget::class);
/** @mir-check $box is Box<Widget> */
===expect===
