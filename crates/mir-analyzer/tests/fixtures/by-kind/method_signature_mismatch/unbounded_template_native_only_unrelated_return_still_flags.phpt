===description===
Negative control: a native-only child whose hint does not admit the bound
template type (an unrelated class) is still reported.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Alpha {}
class Beta {}

/** @template T */
interface Source {
    /** @return T */
    public function get(): object;
}

/** @implements Source<Alpha> */
final class AlphaSource implements Source {
    public function get(): Beta {
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ MethodSignatureMismatch: Method AlphaSource::get() signature mismatch: return type 'Beta' is not a subtype of parent 'Alpha'
        return new Beta();
    }
}
===expect===
