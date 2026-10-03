===description===
Mixin static call should not pollute context
===config===
<mir>
  <issueHandlers>
    <MixedReturnStatement errorLevel="suppress"/>
  </issueHandlers>
</mir>
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
//             ^^^^^^^^^^^^^^^^^^ UndefinedVariable: Variable $__tmp_mixin_var__ is not defined
    }
}
===expect===
