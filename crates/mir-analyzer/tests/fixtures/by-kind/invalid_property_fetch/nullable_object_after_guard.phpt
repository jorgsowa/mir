===description===
InvalidPropertyFetch does NOT fire after a null guard narrows the type.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Foo {
    public string $name = "x";
}

/** @var Foo|null $obj */
$obj = null;
if ($obj !== null) {
    $name = $obj->name;
}
