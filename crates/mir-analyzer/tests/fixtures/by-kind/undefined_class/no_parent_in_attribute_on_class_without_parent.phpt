===description===
No parent in attribute on class without parent
===file===
<?php
#[Attribute]
class SomeAttr
{
    /** @param class-string $class */
    public function __construct(string $class) {}
}

#[SomeAttr(parent::class)]
class A {}

===expect===
ParentNotFound@9:12-9:18: Cannot use parent:: when current class has no parent
