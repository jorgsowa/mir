===description===
A child of a `@psalm-consistent-templates` class must redeclare the parent's templates
in the same order and pass them through unchanged; other generic parents are unaffected.
===config===
<mir>
  <issueHandlers>
    <UnusedClass errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/**
 * @template T
 * @psalm-consistent-constructor
 * @psalm-consistent-templates
 */
class Container {
    /** @var T */
    public $value;

    /** @param T $value */
    public function __construct($value) {
        $this->value = $value;
    }

    /**
     * @template U
     * @param U $value
     * @return static<U>
     */
    public function getInstance($value): static {
        return new static($value);
    }
}

/**
 * @template T
 * @extends Container<T>
 */
class MatchingChild extends Container {}

/**
 * @template TValue
 * @extends Container<TValue>
 */
class RenamedChild extends Container {}

/** @extends Container<string> */
class PinnedChild extends Container {}
//<^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InvalidExtendClass: Class PinnedChild extends Container, which has consistent templates, so it must declare the same template parameters and pass them through unchanged

class BareChild extends Container {}
//<^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InvalidExtendClass: Class BareChild extends Container, which has consistent templates, so it must declare the same template parameters and pass them through unchanged

/**
 * @template T
 * @template U
 * @extends Container<U>
 */
class SwappedChild extends Container {}
//<^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InvalidExtendClass: Class SwappedChild extends Container, which has consistent templates, so it must declare the same template parameters and pass them through unchanged

/**
 * @template T
 * @extends MatchingChild<T>
 */
class GrandChild extends MatchingChild {}

/** @template T */
class Loose {}

/** @extends Loose<string> */
class PinnedLoose extends Loose {}
