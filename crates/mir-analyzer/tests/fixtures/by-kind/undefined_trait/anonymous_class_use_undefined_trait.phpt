===description===
An anonymous class using a nonexistent trait must report UndefinedTrait,
matching a named class's `use` check.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
$x = new class {
    use UndefinedTrait;
//      ^^^^^^^^^^^^^^ UndefinedTrait: Trait UndefinedTrait does not exist
};
