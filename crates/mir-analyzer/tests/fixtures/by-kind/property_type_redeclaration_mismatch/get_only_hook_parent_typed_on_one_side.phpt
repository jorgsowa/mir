===description===
A get-only hook parent still needs a type on both sides or neither; an untyped child of a typed one must be a subtype.
===config===
<mir>
  <phpVersion>8.5</phpVersion>
  <issueHandlers>
    <MissingPropertyType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
abstract class TypedParent {
    abstract public int $x { get; }
}

class UntypedChild extends TypedParent {
    public $x;
//  ^^^^^^^^^^ PropertyTypeRedeclarationMismatch: Type of UntypedChild::$x must be subtype of int (as in class TypedParent)
}

interface UntypedHook {
    public $y { get; }
}

class TypedImplementation implements UntypedHook {
    public int $y = 0;
//  ^^^^^^^^^^^^^^^^^^ PropertyTypeRedeclarationMismatch: Type of TypedImplementation::$y must be omitted to match the parent definition in class UntypedHook
}
