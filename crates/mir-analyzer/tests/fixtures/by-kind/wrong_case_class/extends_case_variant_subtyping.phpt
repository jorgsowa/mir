===description===
PHP class names are case-insensitive: `class Child extends base {}` is the same
class as `Base`. Subtype checks must treat `Child` as a subtype of `Base`
regardless of the case used at the declaration site, in both argument and
return type checks. Before the fix, extends_or_implements compared FQCNs
byte-exact, so the subtype relation was missed and spurious
InvalidArgument / InvalidReturnType issues were emitted. The
WrongCaseClass style diagnostic on the `extends` clause is still reported.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Base {}
class Child extends base {}
//<^^^^^^^^^^^^^^^^^^^^^^^^^^^ WrongCaseClass: Class name 'base' has incorrect casing; use 'Base'
function takes_base(Base $x): void {}
function make(): Base { return new Child(); }
takes_base(new Child());
