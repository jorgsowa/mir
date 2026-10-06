===description===
Variant docblock properties
===config===
<mir>
  <issueHandlers>
    <MissingPropertyType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class ParentClass
{
    /** @var null|string */
    protected $mightExist;
}

class ChildClass extends ParentClass
{
    /** @var string */
    protected $mightExist = "";
}
