===description===
A self-out naming a method template that no parameter mentions (`@template TNewParent` without `@param TNewParent`) retypes the receiver with `mixed` for it, not the template's bound, so a later typed assignment is not rejected.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
namespace App;

interface ParentNode {}

/**
 * @template TParent of ParentNode|null
 */
class Builder
{
    /**
     * @template TNewParent of ParentNode|null
     * @psalm-this-out static<TNewParent>
     * @return $this
     */
    public function setParent(?ParentNode $parent): static
    {
        return $this;
    }
}

final class Tree implements ParentNode
{
    /** @var Builder<static> */
    private Builder $builder;

    public function __construct()
    {
        $builder = new Builder();
        $builder->setParent($this);
        /** @mir-check $builder is App\Builder<mixed> */
        $_ = 1;
        $this->builder = $builder;
    }
}
