===description===
A bare `new Gen()` (nothing to infer the template from) is `Gen<mixed>`, so a
`Gen<int>` property, parameter or return accepts it — same class, a
`@extends`-forwarding subclass, and an uninferable constructor argument.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
    <MissingConstructor errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @template T */
class Gen {
    /** @param T $v */
    public function __construct(public mixed $v = null) {}
}

/**
 * @template T
 * @extends Gen<T>
 */
class Forwarding extends Gen {}

class Holder {
    /** @var Gen<int> */
    public Gen $same;
    /** @var Gen<int> */
    public Gen $sub;

    public function __construct() {
        $this->same = new Gen();
        $this->sub = new Forwarding();
        /** @mir-check $this->same is Gen */
        $x = $this->same;
    }
}

/** @param Gen<int> $g */
function take(Gen $g): void {}

function caller(): void {
    take(new Gen());
    take(new Forwarding());
    $g = new Gen();
    /** @mir-check $g is Gen */
    take($g);
}

/** @return Gen<int> */
function make(): Gen {
    return new Gen();
}
