===description===
Mixin static call should not pollute context
===file===
<?php
/**
 * @template T
 */
class Foo
{
    public function foobar(): void {}
}

/**
 * @template T
 * @mixin Foo<T>
 */
class Bar
{
    public function baz(): self
    {
        self::foobar();
        return $__tmp_mixin_var__;
    }
}
===expect===
UndefinedVariable@19:16-19:34: Variable $__tmp_mixin_var__ is not defined
