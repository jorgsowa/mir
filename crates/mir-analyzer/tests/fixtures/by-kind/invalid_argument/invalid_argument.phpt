===description===
Invalid argument
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
#[Attribute]
class Foo
{
    public function __construct(int $i)
    {
    }
}

#[Foo("foo")]
class Bar{}
===expect===
