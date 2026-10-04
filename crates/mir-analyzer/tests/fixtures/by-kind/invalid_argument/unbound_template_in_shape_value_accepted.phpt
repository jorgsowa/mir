===description===
A class template that has no binding at the call site (a value of type `T`
read from an unparameterised generic) is treated as `mixed` when it sits in an
array, list or shape value, so it fits optional and required shape keys.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @template T */
class Box {
    /** @return T */
    public function get(): mixed { return null; }
}

/** @param array{k?: ?string} $x */
function takeOptional(array $x): void {}

/** @param array{k: ?string} $x */
function takeRequired(array $x): void {}

/** @param array{outer?: array{inner?: int}} $x */
function takeNested(array $x): void {}

/** @param list<?string> $x */
function takeList(array $x): void {}

/** @param array<string, ?string> $x */
function takeMap(array $x): void {}

/**
 * @psalm-type Ctx = array<string, mixed>&array{k?: ?string}
 * @param Ctx $x
 */
function takeIntersection(array $x): void {}

/** @return array{k?: ?string} */
function giveShape(): array { return ['k' => (new Box)->get()]; }

class Holder {
    /** @var array{k?: ?string} */
    public array $prop = [];

    public function fill(): void {
        $this->prop = ['k' => (new Box)->get()];
    }
}

/** @mir-check (new Box)->get() is T */
takeOptional(['k' => (new Box)->get()]);
takeRequired(['k' => (new Box)->get()]);
takeNested(['outer' => ['inner' => (new Box)->get()]]);
takeList([(new Box)->get()]);
takeMap(['k' => (new Box)->get()]);
takeIntersection(['k' => (new Box)->get()]);
===expect===
