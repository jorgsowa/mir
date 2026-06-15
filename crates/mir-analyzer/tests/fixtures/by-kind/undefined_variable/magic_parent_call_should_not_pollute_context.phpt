===description===
Magic parent call should not pollute context
===config===
suppress=MixedReturnStatement
===file===
<?php
/**
 * @method baz(): Foo
 */
class Foo
{
    public function __call()
    {
        return new self();
    }
}

class Bar extends Foo
{
    public function baz(): Foo
    {
        parent::baz();
        return $__tmp_parent_var__;
    }
}
===expect===
UndefinedVariable@18:15-18:34: Variable $__tmp_parent_var__ is not defined
