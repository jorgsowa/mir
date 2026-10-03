===description===
A plain (non-promoted) parameter is only reflectable as a `ReflectionParameter`
— an attribute restricted to TARGET_PROPERTY alone must still be rejected on
it, unlike on a promoted param.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
#[Attribute(Attribute::TARGET_PROPERTY)]
class OnlyProperty {}

function foo(#[OnlyProperty] int $id): void {}
//             ^^^^^^^^^^^^ InvalidAttribute: Attribute OnlyProperty cannot be used on this target
===expect===
