===description===
A redeclared property's `@var` type must match the parent's: wider breaks reads, narrower breaks writes.
===config===
<mir>
  <issueHandlers>
    <MissingPropertyType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
interface ParentInterface {}
interface ChildInterface extends ParentInterface {}

/** @template T */
class Box
{
    /** @var T|null */
    public $value = null;
}

/** @extends Box<ChildInterface> */
class ChildBox extends Box
{
    /** @var ChildInterface|null */
    public $value = null;
}

class Base
{
    /** @var ChildInterface|null */
    public $widened = null;
    /** @var ParentInterface|null */
    public $narrowed = null;
    /** @var ChildInterface|null */
    public $matching = null;
    /** @var list<int> */
    public array $ids = [];
    /** @var int */
    public $untouched = 0;
    /** @var static|null */
    public $self = null;
}

final class Child extends Base
{
    /** @var ParentInterface|null */
    public $widened = null;
//  ^^^^^^^^^^^^^^^^^^^^^^^ NonInvariantDocblockPropertyType: Docblock type 'ParentInterface|null' of Child::$widened is not invariant with 'ChildInterface|null' in class Base
    /** @var ChildInterface|null */
    public $narrowed = null;
//  ^^^^^^^^^^^^^^^^^^^^^^^^ NonInvariantDocblockPropertyType: Docblock type 'ChildInterface|null' of Child::$narrowed is not invariant with 'ParentInterface|null' in class Base
    /** @var ChildInterface|null */
    public $matching = null;
    /** @var list<positive-int> */
    public array $ids = [];
//  ^^^^^^^^^^^^^^^^^^^^^^^ NonInvariantDocblockPropertyType: Docblock type 'list<positive-int>' of Child::$ids is not invariant with 'list<int>' in class Base
    public $untouched = 1;
    /** @var static|null */
    public $self = null;
}
