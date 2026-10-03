===description===
Negative control: a child that restates its own `@return` docblock opts into
that promise, so a mismatch with the bound template is still reported.
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
    /** @return Beta */
    public function get(): object {
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ MethodSignatureMismatch: Method AlphaSource::get() signature mismatch: return type 'Beta' is not a subtype of parent 'Alpha'
        return new Beta();
    }
}
===expect===
