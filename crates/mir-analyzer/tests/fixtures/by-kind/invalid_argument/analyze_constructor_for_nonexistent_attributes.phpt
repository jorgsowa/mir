===description===
Analyze constructor for nonexistent attributes
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Foo
{
    public function __construct(string $_arg) {}
}

/** @suppress UndefinedAttributeClass */
#[AttrA(new Foo(1))]
class Bar {}

===expect===
