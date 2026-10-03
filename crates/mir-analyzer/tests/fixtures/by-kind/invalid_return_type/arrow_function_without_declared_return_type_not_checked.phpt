===description===
D4 negative control: an arrow function with no declared return type (native
or docblock) has nothing to check against — the fix must not invent a
diagnostic when there's no declared type to compare against.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <MissingClosureReturnType errorLevel="suppress"/>
    <MissingPropertyType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Holder {
    /** @var null|string */
    public $name;
}
$f = fn(Holder $h) => $h->name;
===expect===
