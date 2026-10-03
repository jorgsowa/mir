===description===
Magic parent call should not pollute context
===config===
<mir>
  <issueHandlers>
    <MixedReturnStatement errorLevel="suppress"/>
  </issueHandlers>
</mir>
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
//             ^^^^^^^^^^^^^^^^^^^ UndefinedVariable: Variable $__tmp_parent_var__ is not defined
    }
}
===expect===
