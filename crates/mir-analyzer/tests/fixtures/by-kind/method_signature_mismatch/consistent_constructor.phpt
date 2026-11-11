===description===
`@psalm-consistent-constructor` / `@phpstan-consistent-constructor` holds every subclass constructor to the tagged class's constructor signature, so `new static(...)` is safe. A subclass that inherits the constructor, or keeps a compatible one, is accepted.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/**
 * @psalm-consistent-constructor
 */
class Model
{
    public function __construct(string $name)
    {
    }

    public static function make(string $name): static
    {
        return new static($name);
    }
}

final class Inherits extends Model
{
}

final class Compatible extends Model
{
    public function __construct(string $name, int $id = 0)
    {
        parent::__construct($name);
    }
}

final class Incompatible extends Model
{
    public function __construct(int $id)
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ MethodSignatureMismatch: Method Incompatible::__construct() signature mismatch: parameter $id type 'int' is incompatible with parent type 'string'
    {
        parent::__construct((string) $id);
    }
}

class Unchecked
{
    public function __construct(string $name)
    {
    }
}

final class FreeToChange extends Unchecked
{
    public function __construct(int $id)
    {
        parent::__construct((string) $id);
    }
}
