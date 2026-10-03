===description===
A promoted parameter is reflectable as a parameter or a property, but not as
anything else — an attribute restricted to an unrelated target (e.g.
TARGET_CLASS_CONSTANT) must still be rejected.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
#[Attribute(Attribute::TARGET_CLASS_CONSTANT)]
class OnlyClassConstant {}

class Foo {
    public function __construct(
        #[OnlyClassConstant] public int $id,
//        ^^^^^^^^^^^^^^^^^ InvalidAttribute: Attribute OnlyClassConstant cannot be used on this target
    ) {}
}
===expect===
