===description===
Template bounds are not checked against still-unresolved placeholders:
a binding that is itself a template param, self, or static resolves only at
a concrete call site (Eloquent relation pattern). A genuinely wrong concrete
binding still violates.
===file===
<?php
class Model {}

/**
 * @template TRelatedModel of Model
 */
class Relation {
    /**
     * @param TRelatedModel $model
     */
    public function add($model): void {}
}

/**
 * @template TChild of Model
 */
class Builder {
    /** @var Relation<TChild> */
    public $relation;

    /** @param TChild $m */
    public function push($m): void {
        // TChild is unresolved here — must not violate TRelatedModel's bound
        $this->relation->add($m);
    }
}

class NotAModel {}
/** @var Relation<Model> $r */
$r = new Relation();
$r->add(new NotAModel());
===expect===
MissingConstructor@17:0-17:15: Class Builder has uninitialized properties but no constructor
InvalidArgument@31:9-31:24: Argument $model of add() expects 'Model', got 'NotAModel'
