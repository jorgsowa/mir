===description===
No parent in attribute on class without parent
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
#[Attribute]
class SomeAttr
{
    /** @param class-string $class */
    public function __construct(string $class) {}
}

#[SomeAttr(parent::class)]
//         ^^^^^^ ParentNotFound: Cannot use parent:: when current class has no parent
class A {}
