===description===
FN: contravariance/covariance checks were skipped outright whenever the
parent's param/return type mentioned a template, even when this class's own
`@extends Box<int>` concretely bound that template. Substituting the
inherited binding into the ancestor's type before comparing now catches a
real signature narrowing that was previously silent.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/**
 * @template T
 */
class Box {
    /** @param T $x */
    public function set($x): void {}

    /** @return T */
    public function get() {
        return null;
    }
}

/** @extends Box<int> */
class IntBox extends Box {
    public function set(string $x): void {}
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ MethodSignatureMismatch: Method IntBox::set() signature mismatch: parameter $x type 'string' is incompatible with parent type 'int'
}

/** @extends Box<int> */
class StringBox extends Box {
    public function get(): string {
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ MethodSignatureMismatch: Method StringBox::get() signature mismatch: return type 'string' is not a subtype of parent 'int'
        return "x";
    }
}
